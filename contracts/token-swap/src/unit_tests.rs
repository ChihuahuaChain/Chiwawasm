#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{
        to_binary, Addr, Attribute, Decimal, Empty, Env, MemoryStorage, OwnedDeps, Reply, ReplyOn,
        SubMsg, SubMsgResponse, SubMsgResult, WasmMsg,
    };
    use cw20::{Denom, MinterResponse};

    use crate::contract::{instantiate, reply};
    use crate::msg::InstantiateMsg;
    use crate::state::LP_TOKEN;
    use crate::ContractError;

    struct InstantiationResponse {
        deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
        caller: String,
        env: Env,
        msg: InstantiateMsg,
    }

    // todo test instantiation error cases
    #[test]
    fn init_error_identical_denom_not_allowed_in_pair() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("stake")),
            base_denom: Denom::Native(String::from("stake")),
            quote_denom: Denom::Native(String::from("stake")),
            swap_rate: Decimal::from_str("0.3").unwrap(),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::IdenticalDenomNotAllowedInPair {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn init_error_native_token_not_provided_in_pair() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("stake")),
            base_denom: Denom::Native(String::from("woof")),
            quote_denom: Denom::Native(String::from("puppy")),
            swap_rate: Decimal::from_str("0.3").unwrap(),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::NativeTokenNotProvidedInPair {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn init_error_invalid_swap_rate_below_limit() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("stake")),
            base_denom: Denom::Native(String::from("stake")),
            quote_denom: Denom::Native(String::from("puppy")),
            swap_rate: Decimal::from_str("0.05").unwrap(),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::InvalidSwapRate {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn init_error_invalid_swap_rate_above_limit() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("stake")),
            base_denom: Denom::Native(String::from("stake")),
            quote_denom: Denom::Native(String::from("puppy")),
            swap_rate: Decimal::from_str("1.2").unwrap(),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::InvalidSwapRate {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    // This function instantiate the contract and returns reusable components
    fn proper_initialization() -> InstantiationResponse {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("stake")),
            base_denom: Denom::Native(String::from("stake")),
            quote_denom: Denom::Cw20(Addr::unchecked("token_address")),
            swap_rate: Decimal::from_str("0.3").unwrap(),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("instantiate")
            }
        );
        assert_eq!(
            _res.messages[0],
            SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Success,
                msg: WasmMsg::Instantiate {
                    msg: to_binary(&lp_token_info(caller.clone())).unwrap(),
                    code_id: msg.lp_token_code_id,
                    funds: vec![],
                    label: format!("hhslp_{:?}_{:?}", msg.base_denom, msg.quote_denom),
                    admin: None,
                }
                .into()
            }
        );

        // return reusable data
        InstantiationResponse {
            deps,
            caller,
            env,
            msg,
        }
    }

    fn lp_token_info(minter: String) -> cw20_base::msg::InstantiateMsg {
        cw20_base::msg::InstantiateMsg {
            name: "HuahuaSwap LP Token".into(),
            symbol: "hhslpt".into(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse { minter, cap: None }),
            marketing: None,
        }
    }

    #[test]
    fn test_correct_instantiation_reply() {
        let mut _instance = proper_initialization();

        // Test the submsg after cw_20_token is stored
        let contract_addr = String::from("pair0000");
        let reply_msg = Reply {
            id: 0,
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

        // query the contract state to see if the lp contract address was saved
        let lp_token_address = LP_TOKEN.load(&_instance.deps.storage).unwrap();
        assert_eq!(lp_token_address, Addr::unchecked(contract_addr));
    }
}
