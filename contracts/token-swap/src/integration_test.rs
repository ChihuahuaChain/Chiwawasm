#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
    use crate::state::Config;
    use cosmwasm_std::{coins, Addr, Coin, Decimal, Empty, Uint128};
    use cw20::Denom;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const NATIVE_DENOM: &str = "udenom";
    const SUPPLY: u128 = 500_000_000u128;

    // Here we create a struct for instatation config
    struct InstantiationResponse {
        app: App,
        c_template: CwTemplateContract,
        c_addr: Addr,
        msg: InstantiateMsg,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    coins(SUPPLY, NATIVE_DENOM.to_string()),
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

    fn get_contract_info(app: &mut App, contract_address: Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};

        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
    }

    fn mock_instantiate() -> InstantiationResponse {
        let mut app = mock_app();
        let template_id = app.store_code(contract_template());
        let cw20_id = app.store_code(contract_cw20());

        let msg = InstantiateMsg {
            native_denom: Denom::Native(NATIVE_DENOM.to_string()),
            base_denom: Denom::Native(NATIVE_DENOM.to_string()),
            quote_denom: Denom::Native(String::from("puppy")),
            swap_rate: Decimal::from_str("0.3").unwrap(),
            lp_token_code_id: cw20_id,
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

        // return resuable data
        InstantiationResponse {
            app,
            msg,
            c_template: cw_template_contract,
            c_addr: template_contract_addr,
        }
    }

    #[test]
    fn test_instantiate() {
        let mut _instance = mock_instantiate();

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_contract_info(&mut _instance.app, _instance.c_addr);
        assert_eq!(
            info,
            InfoResponse {
                base_denom: Denom::Native(NATIVE_DENOM.to_string()),
                base_reserve: Uint128::zero(),
                quote_reserve: Uint128::zero(),
                quote_denom: Denom::Native(String::from("puppy")),
                swap_rate: Decimal::from_str("0.3").unwrap(),
                lp_token_supply: Uint128::zero(),
                lp_token_address: Addr::unchecked("contract1")
            }
        );
    }
}
