use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub max_balance_to_burn: Uint128,
    pub multiplier: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BurnTokens {
        amount: Uint128,
    },
    UpdatePreferences {
        max_burn_amount: Option<Uint128>,
        multiplier: Option<u8>,
    },
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns Config
    Info {},
}
