#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{
        coins, from_binary, Attribute, BankMsg, Coin, CosmosMsg, Empty, MemoryStorage, OwnedDeps,
        Uint128,
    };

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::ContractError;

    // Here we create a struct for instatation config
    struct InstantiationResponse {
        deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
        owner: String,
    }

    // This function instantiate the contract and returns reusable components
    fn proper_initialization() -> InstantiationResponse {
        let mut deps = mock_dependencies();
        let owner = String::from("creator");
        let community_pool_address = String::from("pool_address");

        let msg = InstantiateMsg {
            owner: Some(owner.clone()),
            community_pool_address,
        };

        // we can just call .unwrap() to assert this was a success
        let funds = coins(1000, "stake");
        let info = mock_info(&owner, &funds);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, _res.messages.len());

        // query and verify state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let contract_config = from_binary(&res).unwrap();
        assert_eq!(msg, contract_config);

        // return reusable data
        InstantiationResponse { deps, owner }
    }

    #[test]
    fn execute_transfer_owner() {
        let mut instance = proper_initialization();

        // create a transfer owner message
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::TransferContractOwnership {
            new_owner: String::from("new_contract_owner"),
        };

        let _res = execute(instance.deps.as_mut(), mock_env(), info, msg).unwrap();
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
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::TransferContractOwnership {
            new_owner: String::from("another_owner"),
        };
        let _err = execute(instance.deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match _err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn execute_burn_balance() {
        let mut instance = proper_initialization();

        // create a burn balance  message
        let msg = ExecuteMsg::BurnContractBalance {};
        let info = mock_info(&instance.owner, &[]);
        let _res = execute(instance.deps.as_mut(), mock_env(), info, msg).unwrap();
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

    // todo ExecuteMsg::BurnDailyQuota
    // todo ExecuteMsg::SetMaxDailyBurn
    // todo ExecuteMsg::WithdrawFundsToCommunityPool
}
