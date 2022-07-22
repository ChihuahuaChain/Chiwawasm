#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Attribute, Uint128, CosmosMsg, BankMsg};

    use crate::contract::{execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::ContractError;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some(String::from("creator")),
            dailyBurnQuota: Uint128::from(1000u128),
            communityPoolAddress: String::from("pool_address"),
        };

        let info = mock_info("creator", &coins(1000, "stake"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, _res.messages.len());
    }

    #[test]
    fn execute_transfer_owner() {
        // setup an instance of the contract
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some(String::from("creator")),
            dailyBurnQuota: Uint128::from(1000u128),
            communityPoolAddress: String::from("pool_address"),
        };

        let info = mock_info("creator", &coins(2, "stake"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create a transfer owner message
        let msg = ExecuteMsg::TransferContractOwnership {
            new_owner: String::from("new_contract_owner"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(_res.attributes.len(), 2);
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("execute_transfer_owner")
            }
        );
        assert_eq!(
            _res.attributes[1],
            Attribute {
                key: String::from("new_owner"),
                value: String::from("new_contract_owner"),
            }
        );

        // Here we try to call transfer owner with the old owner which should fail
        let msg = ExecuteMsg::TransferContractOwnership {
            new_owner: String::from("another_owner"),
        };
        let _err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        match _err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn execute_burn_balance() {
        // setup an instance of the contract
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some(String::from("creator")),
            dailyBurnQuota: Uint128::from(1000u128),
            communityPoolAddress: String::from("pool_address"),
        };

        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create a burn balnce  message
        let msg = ExecuteMsg::BurnContractBalance {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(_res.attributes.len(), 1);
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("execute_burn_balance")
            }
        );

        // check the messages in the response to see if the burn method was called
        assert_eq!(_res.messages.len(), 1);
        assert_eq!(
            _res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Burn { amount: vec![] })
        );
    }
}
