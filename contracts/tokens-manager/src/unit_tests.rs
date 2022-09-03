#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{
        from_binary, to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Empty, Env,
        MemoryStorage, OwnedDeps, Reply, ReplyOn, SubMsg, SubMsgResponse, SubMsgResult, Uint128,
        WasmMsg,
    };
    use cw20::Logo;

    use crate::contract::{execute, instantiate, query, reply};
    use crate::msg::{ExecuteMsg, InstantiateMsg, MarketingInfo, QueryMsg, TokenInfo};
    use crate::state::{entries, Entry, TempEntry, TEMP_ENTRY_STATE};
    use crate::ContractError;

    struct InstantiationResponse {
        deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
        caller: String,
        env: Env,
        msg: InstantiateMsg,
    }

    // This function instantiate the contract and returns reusable components
    fn proper_initialization() -> InstantiationResponse {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("creator");

        let msg = InstantiateMsg {
            token_code_id: 1234u64,
            token_creation_fee: Coin {
                amount: Uint128::from(100000000u128),
                denom: "udenom".to_string(),
            },
        };

        // we can just call .unwrap() to assert this was a success
        let info = mock_info(&caller, &[]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        assert_eq!(0, _res.messages.len());

        // query and verify state
        let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let contract_config = from_binary(&res).unwrap();
        assert_eq!(msg, contract_config);

        // return reusable data
        InstantiationResponse {
            deps,
            caller,
            env,
            msg,
        }
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

    #[test]
    fn test_create_new_token() {
        let mut _instance = proper_initialization();

        // create a new token
        let token_info = new_token_info();

        let info = mock_info(
            &_instance.caller,
            &[_instance.msg.token_creation_fee.clone()],
        );
        let msg = ExecuteMsg::CreateToken {
            token_info: token_info.clone(),
        };

        // Here we call the execute function
        let _res = execute(_instance.deps.as_mut(), _instance.env.clone(), info, msg).unwrap();

        // we can inspect the returned params
        assert_eq!(_res.attributes.len(), 1);
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("execute_create_new_token")
            }
        );
        assert_eq!(
            _res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Burn {
                amount: vec![_instance.msg.token_creation_fee]
            })
        );
        assert_eq!(
            _res.messages[1],
            SubMsg {
                id: 1,
                gas_limit: None,
                reply_on: ReplyOn::Success,
                msg: WasmMsg::Instantiate {
                    msg: to_binary(&token_info).unwrap(),
                    code_id: _instance.msg.token_code_id,
                    funds: vec![],
                    label: token_info.name.clone(),
                    admin: Some(_instance.caller),
                }
                .into()
            }
        );

        // load the TEMP_ENTRY_STATE to see if it contains the right data
        assert_eq!(
            TEMP_ENTRY_STATE.load(&_instance.deps.storage).unwrap(),
            TempEntry {
                name: token_info.name.to_lowercase(),
                symbol: token_info.symbol.to_lowercase(),
                logo: token_info.marketing.logo,
            }
        );
    }

    #[test]
    fn test_create_new_token_reply() {
        let mut _instance = proper_initialization();
        let info = mock_info(
            &_instance.caller,
            &[_instance.msg.token_creation_fee.clone()],
        );
        // create a new token
        let token_info = new_token_info();
        let msg = ExecuteMsg::CreateToken {
            token_info: token_info.clone(),
        };

        // Here we call the execute function
        let _res = execute(_instance.deps.as_mut(), _instance.env.clone(), info, msg).unwrap();

        // Test the submsg after cw_20_token is stored
        let contract_addr = String::from("pair0000");
        let reply_msg = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                // ? derive this line from contract_addr
                data: Some(vec![10, 8, 112, 97, 105, 114, 48, 48, 48, 48].into()),
            }),
        };

        // execute reply message
        let _res = reply(_instance.deps.as_mut(), mock_env(), reply_msg).unwrap();
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("token_contract_addr"),
                value: contract_addr.clone()
            }
        );

        // load the entriesto see if it contains the right data
        let latest_entry = entries()
            .load(
                &_instance.deps.storage,
                token_info.symbol.to_lowercase().as_str(),
            )
            .unwrap();

        assert_eq!(
            latest_entry,
            Entry {
                id: 1u64,
                contract_addr: Addr::unchecked(contract_addr),
                name: token_info.name.to_lowercase(),
                symbol: token_info.symbol.to_lowercase(),
                logo: token_info.marketing.logo,
            }
        );
    }

    #[test]
    fn test_create_new_token_symbol_exists() {
        let mut _instance = proper_initialization();
        let info = mock_info(
            &_instance.caller,
            &[_instance.msg.token_creation_fee.clone()],
        );

        // create a new token
        let token_info = new_token_info();
        let msg = ExecuteMsg::CreateToken {
            token_info: token_info.clone(),
        };
        let _res = execute(
            _instance.deps.as_mut(),
            _instance.env.clone(),
            info.clone(),
            msg,
        )
        .unwrap();

        // Execute reply message
        let reply_msg = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(vec![10, 8, 112, 97, 105, 114, 48, 48, 48, 48].into()),
            }),
        };
        let _res = reply(_instance.deps.as_mut(), mock_env(), reply_msg).unwrap();

        // when we try to create_new_token again with the same token_info, we get an error
        let token_info = new_token_info();
        let msg = ExecuteMsg::CreateToken { token_info };
        let _err = execute(_instance.deps.as_mut(), _instance.env.clone(), info, msg).unwrap_err();

        // we expect the InsufficientContractBalance
        match _err {
            ContractError::TokenWithSymbolAlreadyExists { symbol: _ } => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_create_new_token_with_new_symbol() {
        let mut _instance = proper_initialization();
        let info = mock_info(
            &_instance.caller,
            &[_instance.msg.token_creation_fee.clone()],
        );

        // create a new token
        let token_info = new_token_info();
        let msg = ExecuteMsg::CreateToken {
            token_info: token_info.clone(),
        };
        let _res = execute(
            _instance.deps.as_mut(),
            _instance.env.clone(),
            info.clone(),
            msg,
        )
        .unwrap();

        // Execute reply message
        let reply_msg = Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(vec![10, 8, 112, 97, 105, 114, 48, 48, 48, 48].into()),
            }),
        };
        let _res = reply(_instance.deps.as_mut(), mock_env(), reply_msg).unwrap();

        // when we try to create_new_token with different symbol but the same name, it works
        let mut token_info = new_token_info();
        token_info.symbol = "DFS".to_string();

        let msg = ExecuteMsg::CreateToken { token_info };
        let _res = execute(_instance.deps.as_mut(), _instance.env.clone(), info, msg).unwrap();
    }

    #[test]
    fn test_create_new_token_with_wrong_fee() {
        let incorrect_fee = Coin {
            amount: Uint128::from(200000000u128),
            denom: "udenom".to_string(),
        };

        let mut _instance = proper_initialization();
        let info = mock_info(&_instance.caller, &[incorrect_fee]);

        // create a new token
        let token_info = new_token_info();
        let msg = ExecuteMsg::CreateToken {
            token_info: token_info.clone(),
        };
        let _err = execute(
            _instance.deps.as_mut(),
            _instance.env.clone(),
            info.clone(),
            msg,
        )
        .unwrap_err();

        // we expect the IncorrectTokenCreationFee
        match _err {
            ContractError::IncorrectTokenCreationFee {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }
}
