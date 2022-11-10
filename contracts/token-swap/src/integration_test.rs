#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::str::FromStr;

    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
    use crate::state::Config;
    use crate::ContractError;
    use cosmwasm_std::{coins, Addr, Coin, Decimal, Empty, Uint128};
    use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, Denom, Expiration};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const NATIVE_DENOM: &str = "udenom";
    const NON_DENOM_COIN: &str = "urandom";
    const SUPPLY: u128 = 500_000_000u128;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                        Coin {
                            denom: NON_DENOM_COIN.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                    ],
                )
                .unwrap();
        })
    }

    fn contract_cw20() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        ))
    }

    fn contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_reply(crate::contract::reply),
        )
    }

    fn get_amm_contract_info(app: &mut App, contract_address: &Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};

        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
    }

    // CreateCW20 create new cw20 with given initial balance belonging to owner
    fn create_cw20_quote_token(
        router: &mut App,
        owner: &Addr,
        name: String,
        symbol: String,
        balance: Uint128,
    ) -> Cw20Contract {
        // set up cw20 contract with some tokens
        let cw20_id = router.store_code(contract_cw20());
        let msg = cw20_base::msg::InstantiateMsg {
            name,
            symbol,
            decimals: 2,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount: balance,
            }],
            mint: None,
            marketing: None,
        };
        let addr = router
            .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "CASH", None)
            .unwrap();
        Cw20Contract(addr)
    }

    fn _instantiate_amm(app: &mut App, quote_token_addr: Addr) -> Addr {
        let template_id = app.store_code(contract_template());
        let lp_code_id = app.store_code(contract_cw20());

        let msg = InstantiateMsg {
            native_denom: Denom::Native(NATIVE_DENOM.to_string()),
            base_denom: Denom::Native(NATIVE_DENOM.to_string()),
            quote_denom: Denom::Cw20(quote_token_addr),
            swap_rate: Decimal::from_str("0.3").unwrap(),
            lp_token_code_id: lp_code_id,
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "token_swap",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(template_contract_addr.clone());

        // return addr
        template_contract_addr
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut app,
            &Addr::unchecked(USER),
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(500_000_000),
        );

        let mut amm_addr = _instantiate_amm(&mut app, quote_token_contract.addr());

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_amm_contract_info(&mut app, &amm_addr);

        assert_eq!(
            info,
            InfoResponse {
                base_denom: Denom::Native(NATIVE_DENOM.to_string()),
                base_reserve: Uint128::zero(),
                quote_reserve: Uint128::zero(),
                quote_denom: Denom::Cw20(quote_token_contract.addr()),
                swap_rate: Decimal::from_str("0.3").unwrap(),
                lp_token_supply: Uint128::zero(),
                lp_token_address: Addr::unchecked("contract2")
            }
        );
    }

    #[test]
    fn test_add_liquidity() {
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut router,
            &owner,
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(5000),
        );

        // amm contract instance
        let amm_addr = _instantiate_amm(&mut router, quote_token_contract.addr());

        // make sure that quote_token_contract.addr() != amm_addr
        assert_ne!(quote_token_contract.addr(), amm_addr);

        // Query amm info
        let info = get_amm_contract_info(&mut router, &amm_addr);

        // set up cw20 helpers
        let lp_token = Cw20Contract(Addr::unchecked(info.lp_token_address));

        // check quote_token balance for owner
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(5000));

        // increase the spending allowance of the amm_contract on the quote_token_contract
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(100u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // ContractError::MsgExpirationError {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: Some(Expiration::AtHeight(0)),
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // ContractError::NonZeroInputAmountExpected {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(0),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // ContractError::InsufficientFunds {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(50),
                }],
            )
            .unwrap_err();

        // ContractError::InsufficientFunds {} also when we send NON_DENOM_COIN as funds
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NON_DENOM_COIN.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // ContractError::MaxQuoteTokenAmountExceeded {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(80),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // Add initial liquidity proper and ensure balances are updated ===============================>
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4900));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(100));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(100));

        // Top-up liquidity and ensure balances are updated ===============================>
        // increase the spending allowance of the amm_contract on the quote_token_contract
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(50u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(50),
            max_quote_token_amount: Uint128::new(50),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(50),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4850));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(150));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(150));
    }
}
