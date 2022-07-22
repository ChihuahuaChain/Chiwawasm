use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub burn_delay_in_seconds: u64,
    pub community_pool_address: Addr,
    pub daily_burn_quota: Uint128,
    pub owner: Addr,
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");

// This stores the time when the BurnDailyQuota method is ready to be called
pub const BURN_READY_TIMESTAMP: Item<Timestamp> = Item::new("burn_ready_timestamp");