use cosmwasm_std::{Addr, Uint128, Decimal};
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
    pub swap_rate: Decimal,
}

pub const LP_TOKEN: Item<Addr> = Item::new("lp_token");
pub const BASE_TOKEN: Item<Token> = Item::new("base_token");
pub const QUOTE_TOKEN: Item<Token> = Item::new("quote_token");
pub const CONFIG: Item<Config> = Item::new("config");
