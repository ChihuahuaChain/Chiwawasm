use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");
