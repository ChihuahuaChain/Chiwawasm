use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub community_pool_address: Addr,
    pub daily_burn_amount: u128,
    pub owner: Addr,
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");

// This stores the time when the BurnDailyQuota method is ready to be called
pub const BURN_READY_TIMESTAMP: Item<Timestamp> = Item::new("burn_ready_timestamp");

// This stores the burn_delay in seconds
pub const BURN_DELAY_SECONDS: u64 = 86400u64;
