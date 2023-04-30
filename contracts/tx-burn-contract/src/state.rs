use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub max_extra_balance_to_burn_per_tx: Uint128,
    pub multiplier: u8,
    pub total_amount_burned: Uint128,
    pub total_tx_burned: u64,
    pub total_balance_burned: Uint128,
}

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");
