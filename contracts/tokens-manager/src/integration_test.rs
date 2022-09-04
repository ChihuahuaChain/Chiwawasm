#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MarketingInfo, TokenInfo};

    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw20::Logo;
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

    fn contract_cw20() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        ))
    }

    fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        let contract_with_reply = contract.with_reply(crate::contract::reply);
        Box::new(contract_with_reply)
    }

    fn new_token_info() -> TokenInfo {
        TokenInfo {
            name: "Test Token".to_string(),
            symbol: "TTT".to_string(),
            decimals: 6u8,
            initial_balances: vec![],
            mint: None,
            marketing: MarketingInfo {
                project: "Test Test Test".to_string(),
                description: "Testing token".to_string(),
                marketing: "Test! Test! Test!".to_string(),
                logo: Logo::Url("logo_url".to_string()),
            },
        }
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    // this amount denote the chain total supply
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::from(SUPPLY),
                    }],
                )
                .unwrap();
        })
    }

    fn mock_instantiate() -> InstantiationResponse {
        let mut app = mock_app();
        let template_id = app.store_code(contract_template());
        let cw20_id = app.store_code(contract_cw20());

        let msg = InstantiateMsg {
            token_code_id: cw20_id,
            token_creation_fee: Coin {
                amount: Uint128::from(100_000_000u128),
                denom: "udenom".to_string(),
            },
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "tokens_manager",
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
    fn test_create_new_token_flow() {
        /*let mut _instance = mock_instantiate();

        // here we call the create_new_token
        let token_info = new_token_info();

        let _res = _instance
            .app
            .execute_contract(
                Addr::unchecked(USER),
                _instance.c_addr,
                &ExecuteMsg::CreateToken { token_info },
                &[_instance.msg.token_creation_fee],
            )
            .unwrap();*/
    }
}
