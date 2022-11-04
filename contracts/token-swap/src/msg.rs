use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::{Denom, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Note: This contract only handles pairing HUAHUA to cw20
// we can add support for IBC tokens as well
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // {"native":"udenom"}
    pub native_denom: Denom,
    pub base_denom: Denom,
    pub quote_denom: Denom,
    pub lp_token_code_id: u64,
    pub swap_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenSelect {
    Base,
    Quote,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ///
    AddLiquidity {
        token1_amount: Uint128,

        // Q?
        min_liquidity: Uint128,

        // Q?
        max_token2: Uint128,

        // Q?
        expiration: Option<Expiration>,
    },

    ///
    RemoveLiquidity {
        amount: Uint128,

        // Q?
        min_token1: Uint128,
        min_token2: Uint128,
        expiration: Option<Expiration>,
    },

    ///
    Swap {
        input_token: TokenSelect,
        input_amount: Uint128,

        // Q?
        min_output: Uint128,

        expiration: Option<Expiration>,
    },

    /// Chained swap converting A -> B and B -> C by leveraging two swap contracts
    PassThroughSwap {
        // Q?
        output_amm_address: Addr,
        input_token: TokenSelect,
        input_token_amount: Uint128,
        output_min_token: Uint128,
        expiration: Option<Expiration>,
    },

    SwapAndSendTo {
        input_token: TokenSelect,
        input_amount: Uint128,
        recipient: Addr,
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance { address: String },

    /// Returns information about the current state of the pool
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub base_reserve: Uint128,
    pub base_denom: Denom,
    pub quote_reserve: Uint128,
    pub quote_denom: Denom,
    pub swap_rate: Decimal,
    pub lp_token_supply: Uint128,
    pub lp_token_address: Addr,
}
