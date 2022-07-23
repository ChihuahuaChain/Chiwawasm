#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info, MockApi,
        MockQuerier,
    };
    use cosmwasm_std::{
        coins, from_binary, Attribute, BalanceResponse, BankMsg, Coin, CosmosMsg, Empty, Env,
        MemoryStorage, OwnedDeps, Timestamp, Uint128,
    };

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{BURN_DELAY_SECONDS, DEFAULT_DAILY_QUOTA};
    use crate::ContractError;

    const NATIVE_DENOM: &str = "udenom";

    // Here we create a struct for instatation config
    struct InstantiationResponse {
        deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
        owner: String,
        env: Env,
    }

    // This function instantiate the contract and returns reusable components
    fn proper_initialization(contract_balances: &[Coin]) -> InstantiationResponse {
        let mut deps = mock_dependencies_with_balance(contract_balances);
        let env = mock_env();
        let owner = String::from("creator");
        let community_pool_address = String::from("pool_address");

        let msg = InstantiateMsg {
            owner: Some(owner.clone()),
            community_pool_address,
            native_denom: String::from(NATIVE_DENOM),
        };

        // we can just call .unwrap() to assert this was a success
        let info = mock_info(&owner, &[]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        assert_eq!(0, _res.messages.len());

        // query and verify state
        let res = query(deps.as_ref(), env.clone(), QueryMsg::GetConfig {}).unwrap();
        let contract_config = from_binary(&res).unwrap();
        assert_eq!(msg, contract_config);

        // query and verify balance
        let res = query(deps.as_ref(), env.clone(), QueryMsg::QueryBalance {}).unwrap();
        let balance: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(
            balance,
            BalanceResponse {
                amount: match contract_balances.len() > 0 {
                    true => contract_balances[0].clone(),
                    false => Coin {
                        amount: Uint128::from(0u128),
                        denom: String::from(NATIVE_DENOM)
                    },
                }
            },
        );

        // return reusable data
        InstantiationResponse { deps, owner, env }
    }

    #[test]
    fn execute_transfer_owner() {
        let funds: [Coin; 0] = [];
        let mut instance = proper_initialization(&funds);

        // create a transfer owner message
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::TransferContractOwnership {
            new_owner: String::from("new_contract_owner"),
        };

        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();
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
        let _err = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap_err();
        match _err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn execute_burn_balance() {
        let funds: [Coin; 0] = [];
        let mut instance = proper_initialization(&funds);

        // create a burn balance  message
        let msg = ExecuteMsg::BurnContractBalance {};
        let info = mock_info(&instance.owner, &[]);
        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();
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

    #[test]
    fn execute_burn_daily_quota() {
        // we instantiate the contract with a balance > than the daily quota
        const EXTRA_FUNDS: u128 = 123456u128;
        let funds = coins(DEFAULT_DAILY_QUOTA + EXTRA_FUNDS, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

        // Here we set the block time
        instance.env.block.time = Timestamp::from_seconds(0);

        // TEST CASE 1;
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::BurnDailyQuota {};

        // when called the first time, it should burn the daily quota
        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();

        // we can inspect the returned params
        assert_eq!(_res.attributes.len(), 1);
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("execute_burn_daily_quota")
            }
        );
        assert_eq!(_res.messages.len(), 1);
        assert_eq!(
            _res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Burn {
                amount: vec![Coin {
                    denom: String::from(NATIVE_DENOM),
                    amount: Uint128::from(DEFAULT_DAILY_QUOTA),
                }]
            })
        );

        // we then query the contract balance to see if it tallies with expectation
        let msg = QueryMsg::QueryBalance {};
        let res = query(instance.deps.as_ref(), instance.env.clone(), msg).unwrap();
        let balance: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(
            balance,
            BalanceResponse {
                amount: Coin {
                    amount: Uint128::from(EXTRA_FUNDS),
                    denom: String::from(NATIVE_DENOM),
                },
            },
        );

        // TEST CASE 2;
        let msg = ExecuteMsg::BurnDailyQuota {};
        let info = mock_info(&instance.owner, &[]);

        // when called again it should not allow us to burn
        // this is because the next burn duration has not been reached
        let _err = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap_err();
        match _err {
            ContractError::DailyBurnNotReady {} => {}
            e => panic!("unexpected error: {}", e),
        }

        // TEST CASE 3;
        let msg = ExecuteMsg::BurnDailyQuota {};
        let info = mock_info(&instance.owner, &[]);

        // we can fast foward the time by BURN_DELAY_SECONDS + 3600seconds
        instance.env.block.time =
            Timestamp::from_seconds(0).plus_seconds(BURN_DELAY_SECONDS + 3600);

        // it should allow us to call the burn function again and beacuse contract balance
        // is now less than dailyQuota, it should burn all balance
        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();
        // we can inspect the returned params
        assert_eq!(_res.attributes.len(), 1);
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("execute_burn_daily_quota")
            }
        );
        assert_eq!(_res.messages.len(), 1);
        assert_eq!(
            _res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Burn {
                amount: vec![Coin {
                    denom: String::from(NATIVE_DENOM),
                    amount: Uint128::from(EXTRA_FUNDS),
                }]
            })
        );

        // query the balance again to ensure its zero
        let msg = QueryMsg::QueryBalance {};
        let res = query(instance.deps.as_ref(), instance.env.clone(), msg).unwrap();
        let balance: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(
            balance,
            BalanceResponse {
                amount: Coin {
                    amount: Uint128::from(0u128),
                    denom: String::from(NATIVE_DENOM),
                },
            },
        );

        // TEST CASE 4;
        // we can verify this by calling the burn method again which should return a contract
        // error stating there is no tokens to burn
        let msg = ExecuteMsg::BurnDailyQuota {};
        let info = mock_info(&instance.owner, &[]);

        // when called again it should not allow us to burn
        // this is because the next burn duration has not been reached
        let _err = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap_err();
        match _err {
            ContractError::InsufficientContractBalance {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    // todo ExecuteMsg::SetMaxDailyBurn
    // todo ExecuteMsg::WithdrawFundsToCommunityPool
}
