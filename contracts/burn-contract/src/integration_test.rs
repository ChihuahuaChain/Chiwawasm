#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

    use cosmwasm_std::{Addr, BalanceResponse, BlockInfo, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        let contract_with_sudo = contract.with_sudo(crate::contract::sudo);
        Box::new(contract_with_sudo)
    }

    const NATIVE_DENOM: &str = "udenom";
    const DEFAULT_DAILY_QUOTA: u128 = 500_000_000_000_000u128;
    const BURN_DELAY_SECONDS: u64 = 86400u64;
    const USER: &str = "user";

    // Here we create a struct for instatation config
    struct InstantiationResponse {
        app: App,
        c_template: CwTemplateContract,
        c_addr: Addr,
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
                        amount: Uint128::from(DEFAULT_DAILY_QUOTA * 100),
                    }],
                )
                .unwrap();
        })
    }

    fn mock_instantiate(funds: &[Coin]) -> InstantiationResponse {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let community_pool_address = String::from("community_pool_address");
        let burn_delay_seconds = BURN_DELAY_SECONDS;
        let daily_burn_amount = Uint128::from(DEFAULT_DAILY_QUOTA);

        let msg = InstantiateMsg {
            community_pool_address,
            burn_delay_seconds,
            daily_burn_amount,
            native_denom: String::from(NATIVE_DENOM),
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(USER),
                &msg,
                funds, // set a contract balance
                "cw-unity-prop",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr.clone());

        // return resuable data
        InstantiationResponse {
            app,
            c_template: cw_template_contract,
            c_addr: cw_template_contract_addr,
        }
    }

    fn advance_one_hour_after_delay(block: &mut BlockInfo) {
        let one_day_one_hour_in_seconds = BURN_DELAY_SECONDS + 3600;
        block.time = block.time.plus_seconds(one_day_one_hour_in_seconds);
        // av of 1 per 5s
        let blocks_to_advance = one_day_one_hour_in_seconds / 5;
        block.height += blocks_to_advance;
    }

    fn get_balance(app: &mut App, contract_address: Addr) -> BalanceResponse {
        let msg = QueryMsg::Balance {};
        let result: BalanceResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
    }

    #[test]
    fn execute_burn_daily_quota() {
        // create a contract instance with funds greater than the daily burn amount
        const EXTRA_FUNDS: u128 = 123456u128;
        let funds = [Coin {
            denom: String::from(NATIVE_DENOM),
            amount: Uint128::from(DEFAULT_DAILY_QUOTA + EXTRA_FUNDS),
        }];
        let mut instance = mock_instantiate(&funds);

        // Here we call the burn daily quota
        let sender = Addr::unchecked(USER);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let cosmos_msg = instance.c_template.call(msg).unwrap();
        instance.app.execute(sender, cosmos_msg).unwrap();

        // we check to see if the contract balance has EXTRA_FUNDS left
        let contract_balance = get_balance(&mut instance.app, instance.c_addr.clone());
        assert_eq!(
            contract_balance,
            BalanceResponse {
                amount: Coin {
                    denom: String::from(NATIVE_DENOM),
                    amount: Uint128::from(EXTRA_FUNDS),
                },
            }
        );

        // now we call the contract again and this time it should return error
        // due to DailyBurnNotReady
        let sender = Addr::unchecked(USER);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let cosmos_msg = instance.c_template.call(msg).unwrap();
        instance.app.execute(sender, cosmos_msg).unwrap_err();

        // we fast foward the chain to after the burn_delay_time and this time we should be able to burn
        // we inspect the balance to make sure it is now zero
        let sender = Addr::unchecked(USER);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let cosmos_msg = instance.c_template.call(msg).unwrap();

        instance.app.update_block(advance_one_hour_after_delay);
        instance.app.execute(sender, cosmos_msg).unwrap();

        let contract_balance = get_balance(&mut instance.app, instance.c_addr);
        assert_eq!(
            contract_balance,
            BalanceResponse {
                amount: Coin {
                    denom: String::from(NATIVE_DENOM),
                    amount: Uint128::from(0u128),
                },
            }
        );

        // We fast foward the time and try to burn again
        // this time it erors due to InsufficientContractBalance
        let sender = Addr::unchecked(USER);
        let msg = ExecuteMsg::BurnDailyQuota {};
        let cosmos_msg = instance.c_template.call(msg).unwrap();

        instance.app.update_block(advance_one_hour_after_delay);
        instance.app.execute(sender, cosmos_msg).unwrap_err();
    }
}
