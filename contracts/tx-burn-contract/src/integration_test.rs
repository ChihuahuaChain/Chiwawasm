#[cfg(test)]
mod tests {
    use crate::{
        msg::{InstantiateMsg, QueryMsg},
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

    fn instantiate_accounts_manager(app: &mut App) -> Addr {
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
        let result: Config = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();
        let amm_addr = instantiate_accounts_manager(&mut app);

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_contract_info(&mut app, &amm_addr);

        assert_eq!(
            info,
            Config {
                max_balance_to_burn: Uint128::new(100_000_000),
                multiplier: 2u8,
                balance_burned_already: Uint128::zero()
            }
        );
    }
}
