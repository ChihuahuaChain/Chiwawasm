#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::Config,
    };
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const STAKING_DENOM: &str = "TOKEN";
    const SUPPLY: u128 = 500_000_000u128;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: STAKING_DENOM.to_string(),
                        amount: Uint128::from(SUPPLY),
                    }],
                )
                .unwrap();
        })
    }

    fn contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn instantiate_contract(app: &mut App) -> Addr {
        let template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            max_balance_to_burn: Uint128::new(100_000_000),
            multiplier: 2u8,
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "accounts_manager",
                None,
            )
            .unwrap();

        // return addr
        template_contract_addr
    }

    fn get_contract_info(app: &mut App, contract_address: &Addr) -> Config {
        let msg = QueryMsg::Info {};
        app.wrap().query_wasm_smart(contract_address, &msg).unwrap()
    }

    fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> Coin {
        router
            .wrap()
            .query_balance(addr.to_string(), denom)
            .unwrap()
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();
        let amm_addr = instantiate_contract(&mut app);

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_contract_info(&mut app, &amm_addr);

        assert_eq!(
            info,
            Config {
                admin: Addr::unchecked(USER),
                max_balance_to_burn: Uint128::new(100_000_000),
                multiplier: 2u8,
                balance_burned_already: Uint128::zero()
            }
        );
    }

    #[test]
    fn test_update_preferences() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let contract_addr = instantiate_contract(&mut router);

        // Step 2
        // Update preferences
        // ------------------------------------------------------------------------------
        let new_max_burn_amount = Uint128::new(200_000_000);
        let execute_msg = ExecuteMsg::UpdatePreferences {
            max_burn_amount: Some(new_max_burn_amount),
            multiplier: None,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &execute_msg,
                &[],
            )
            .unwrap();

        // Step 3
        // query contract info to verify that values checks out
        // ------------------------------------------------------------------------------
        let info = get_contract_info(&mut router, &contract_addr);
        assert_eq!(
            info,
            Config {
                admin: Addr::unchecked(USER),
                balance_burned_already: Uint128::zero(),
                max_balance_to_burn: new_max_burn_amount,
                multiplier: 2u8,
            }
        );
    }

    #[test]
    fn test_withdraw_balance() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let contract_addr = instantiate_contract(&mut router);

        // Step 2
        // Send some tokens to contract_addr
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: amount,
            },
        };
        router
            .execute_contract(
                wrong_owner,
                contract_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Test error case ContractError::InsufficientBalance {}
        // ------------------------------------------------------------------------------
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: amount + amount,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Withdraw half of the contract balance without providing an optional recipient
        // ------------------------------------------------------------------------------
        let half = Uint128::new(500_000);
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: half,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 6
        // Verify caller's balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(
            &mut router,
            &Addr::unchecked(USER),
            STAKING_DENOM.to_string(),
        );
        assert_eq!(balance.amount, Uint128::new(SUPPLY) - half);

        // Step 7
        // Withdraw the remaining half of the contract balance
        // by providing an optional recipient
        // ------------------------------------------------------------------------------
        let recipient = Addr::unchecked("recipient");
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: Some(recipient.to_string()),
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: half,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 8
        // Verify recipient's balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &recipient, STAKING_DENOM.to_string());
        assert_eq!(balance.amount, half);

        // Step 9
        // Verify that the contract_addr balance is zero
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &contract_addr, STAKING_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::zero());
    }

    #[test]
    fn test_burn_tokens() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let contract_addr = instantiate_contract(&mut router);

        // Step 2
        // Send some tokens to contract_addr
        // ------------------------------------------------------------------------------
        let amount_sent_to_contract = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount: amount_sent_to_contract,
                }],
            )
            .unwrap();

        // Step 3
        // Test error case ContractError::IncorrectAmountProvided {}
        // ------------------------------------------------------------------------------
        let burn_token_msg = ExecuteMsg::BurnTokens {
            amount: amount_sent_to_contract,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &burn_token_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Send 100_000_000 to be burned by the contract
        // ------------------------------------------------------------------------------
        let amount_to_burn = Uint128::new(100_000_000);
        let burn_token_msg = ExecuteMsg::BurnTokens {
            amount: amount_to_burn,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &burn_token_msg,
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount: amount_to_burn,
                }],
            )
            .unwrap();

        // Step 5
        // Verify that the contract_addr balance is zero
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &contract_addr, STAKING_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::zero());

        // Step 6
        // Verify that the user balance has decreased 
        // by amount_sent_to_contract + amount_to_burn
        // ------------------------------------------------------------------------------
        let balance = bank_balance(
            &mut router,
            &Addr::unchecked(USER),
            STAKING_DENOM.to_string(),
        );
        assert_eq!(
            balance.amount,
            Uint128::from(SUPPLY) - amount_sent_to_contract - amount_to_burn
        );
    }
}
