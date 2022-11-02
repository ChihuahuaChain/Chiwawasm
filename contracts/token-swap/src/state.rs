use cosmwasm_std::{Addr, Uint128};
use cw20::Denom;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub reserve: Uint128,
    pub denom: Denom,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub burn_rate: u64,
}

pub const LP_TOKEN: Item<Addr> = Item::new("lp_token");
pub const TOKEN1: Item<Token> = Item::new("token1");
pub const TOKEN2: Item<Token> = Item::new("token2");
pub const CONFIG: Item<Config> = Item::new("config");
