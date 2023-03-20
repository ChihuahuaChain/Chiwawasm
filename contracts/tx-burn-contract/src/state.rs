use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // todo
    // amount_burned_already
    // max_balance_to_burn
    // multiplier
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");
