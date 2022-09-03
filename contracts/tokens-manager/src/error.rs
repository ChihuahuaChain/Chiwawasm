use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Incorrect amount sent as token_creation_fee")]
    IncorrectTokenCreationFee {},

    #[error("Token with symbol: {symbol:?} already exists")]
    TokenWithSymbolAlreadyExists { symbol: String },

    #[error("Token with name: {name:?} already exists")]
    TokenWithNameAlreadyExists { name: String },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
