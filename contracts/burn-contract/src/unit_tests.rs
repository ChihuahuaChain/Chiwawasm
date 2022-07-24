#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_dependencies_with_balance, mock_env, mock_info, MockApi, MockQuerier,
    };
    use cosmwasm_std::{
        coins, from_binary, Attribute, BankMsg, Coin, CosmosMsg, Empty, Env, MemoryStorage,
        OwnedDeps, Uint128,
    };

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::ContractError;

    const NATIVE_DENOM: &str = "udenom";
    const DEFAULT_DAILY_QUOTA: u128 = 500_000_000_000_000u128;
    const BURN_DELAY_SECONDS: u64 = 86400u64;

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
            community_pool_address,
            native_denom: String::from(NATIVE_DENOM),
            daily_burn_amount: Uint128::from(DEFAULT_DAILY_QUOTA),
            burn_delay_seconds: BURN_DELAY_SECONDS,
        };

        // we can just call .unwrap() to assert this was a success
        let info = mock_info(&owner, &[]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        assert_eq!(0, _res.messages.len());

        // query and verify state
        let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let contract_config = from_binary(&res).unwrap();
        assert_eq!(msg, contract_config);

        // return reusable data
        InstantiationResponse { deps, owner, env }
    }

    #[test]
    fn burn_daily_quota_succeed_with_balance_bigger_than_daily_limit() {
        const EXTRA_FUNDS: u128 = 123456u128;
        let funds = coins(DEFAULT_DAILY_QUOTA + EXTRA_FUNDS, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

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
    }

    #[test]
    fn burn_daily_quota_succeed_with_balance_smaller_than_daily_limit() {
        const EXTRA_FUNDS: u128 = 123456u128;
        let funds = coins(EXTRA_FUNDS, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

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
                    amount: Uint128::from(EXTRA_FUNDS),
                }]
            })
        );
    }

    #[test]
    fn burn_daily_quota_failed_when_called_before_burn_time() {
        const EXTRA_FUNDS: u128 = 123456u128;
        let funds = coins(EXTRA_FUNDS, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

        // when called the first time, it should burn the daily quota
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();
        assert_eq!(_res.messages.len(), 1);

        // when called again it should not allow us to burn
        // this is because the next burn duration has not been reached
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let _err = execute(instance.deps.as_mut(), instance.env, info, msg).unwrap_err();
        match _err {
            ContractError::DailyBurnNotReady {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn burn_daily_quota_succeed_when_called_after_burn_time() {
        let funds = coins(DEFAULT_DAILY_QUOTA, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

        // when called the first time, it should burn the daily quota
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let _res = execute(instance.deps.as_mut(), instance.env.clone(), info, msg).unwrap();
        assert_eq!(_res.messages.len(), 1);

        // we can fast foward the time by BURN_DELAY_SECONDS + 3600seconds
        instance.env.block.time = instance
            .env
            .block
            .time
            .plus_seconds(BURN_DELAY_SECONDS + 3600);

        // it should allow us to call the burn function again and beacuse contract balance
        // is now less than dailyQuota, it should burn all balance
        let info = mock_info(&instance.owner, &[]);
        let msg = ExecuteMsg::BurnDailyQuota {};
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
    }

    #[test]
    fn execute_burn_daily_quota_fail_with_zero_balance() {
        let funds = coins(0u128, NATIVE_DENOM);
        let mut instance = proper_initialization(&funds);

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

    #[test]
    fn execute_burn_daily_quota_fail_with_no_balance() {
        let mut instance = proper_initialization(&[]);

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

    // todo SudoMsg::SetMaxDailyBurn
    // todo SudoMsg::WithdrawFundsToCommunityPool
}
