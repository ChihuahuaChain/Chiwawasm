#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, MarketingInfo, QueryMsg, TokenInfo, TokenListResponse,
    };
    use crate::state::{Config, Entry};

    use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
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
        Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_reply(crate::contract::reply),
        )
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
                marketing: "test".to_string(),
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
                    coins(SUPPLY, NATIVE_DENOM.to_string()),
                )
                .unwrap();
        })
    }

    fn get_token_list(app: &mut App, contract_address: Addr) -> TokenListResponse {
        let msg = QueryMsg::QueryTokenList {
            start_after: None,
            limit: None,
        };
        let result: TokenListResponse =
            app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
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
        let mut _instance = mock_instantiate();

        // here we call the create_new_token
        let token_info = new_token_info();
        let _res = _instance
            .app
            .execute_contract(
                Addr::unchecked(USER),
                _instance.c_addr.clone(),
                &ExecuteMsg::CreateToken { token_info },
                &[_instance.msg.token_creation_fee],
            )
            .unwrap();

        // verify that the new token is stored in the returned list
        let list = get_token_list(&mut _instance.app, _instance.c_addr);
        assert_eq!(
            list,
            TokenListResponse {
                entries: vec![Entry {
                    id: 1,
                    name: "test token".to_string(),
                    symbol: "ttt".to_string(),
                    logo: Logo::Url("logo_url".to_string()),
                    contract_addr: Addr::unchecked("contract1")
                }]
            }
        );
    }
}
