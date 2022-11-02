use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Cw20Error(#[from] cw20_base::ContractError),

    #[error("None Error")]
    NoneError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Min liquidity error: requested: {min_liquidity}, available: {liquidity_available}")]
    MinLiquidityError {
        min_liquidity: Uint128,
        liquidity_available: Uint128,
    },

    #[error("Max token error: max_token: {max_token}, tokens_required: {tokens_required}")]
    MaxTokenError {
        max_token: Uint128,
        tokens_required: Uint128,
    },

    #[error("Insufficient liquidity error: requested: {requested}, available: {available}")]
    InsufficientLiquidityError {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Min token1 error: requested: {requested}, available: {available}")]
    MinToken1Error {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Min token2 error: requested: {requested}, available: {available}")]
    MinToken2Error {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Incorrect native denom: provided: {provided}, required: {required}")]
    IncorrectNativeDenom { provided: String, required: String },

    #[error("Swap min error: min: {min}, available: {available}")]
    SwapMinError { min: Uint128, available: Uint128 },

    #[error("MsgExpirationError")]
    MsgExpirationError {},

    #[error("InsufficientFunds")]
    InsufficientFunds {},

    #[error("Uknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Failed to instantiate lp token")]
    InstantiateLpTokenError {},

    #[error("Burn Rate is missing")]
    BurnRateIsMissing {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
