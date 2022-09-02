use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo, TokenListResponse};
use crate::state::{entries, Config, DEFAULT_LIMIT, INIT_CONFIG, MAX_LIMIT};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw_storage_plus::Bound;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        token_creation_fee: msg.token_creation_fee,
    };

    // save the owner to the INIT_CONFIG state
    INIT_CONFIG.save(deps.storage, &config)?;

    // return response
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _: Env,
    msg_info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateToken { token_info } => {
            execute_create_new_token(deps, msg_info, token_info)
        }
    }
}

fn execute_create_new_token(
    deps: DepsMut,
    info: MessageInfo,
    token_info: TokenInfo,
) -> Result<Response, ContractError> {
    let config = INIT_CONFIG.load(deps.storage)?;

    // Check if the token_info is valid
    token_info.validate()?;

    // Check if this token does not already exist
    let empty = entries().may_load(deps.storage, (&token_info.name, &token_info.symbol))?;
    if let Some(d) = empty {
        return Err(ContractError::TokenAlreadyExists {
            name: token_info.name,
            symbol: token_info.symbol,
        });
    }

    // Check if the amount sent by the caller is equal to the token_creation_fee
    let coin = info.funds.iter().find(|coin| {
        coin.denom == config.token_creation_fee.denom
            && coin.amount == config.token_creation_fee.amount
    });

    let fee = match coin {
        Some(coin) => coin.clone(),
        None => {
            return Err(ContractError::IncorrectTokenCreationFee {});
        }
    };

    // Add message to burn the token_creation_fee
    let amount = vec![fee];
    let burn_msg = BankMsg::Burn { amount };
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    // todo add logic that calls the third party contract to create new token

    // Build response
    let res = Response::new()
        .add_attribute("method", "execute_create_new_token")
        .add_messages(msgs);

    // return response
    Ok(res)
}

// todo add logic to handle reply from creating token and get the contract address

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Match and route the query message to the appropriate handler
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::QueryTokenList { start_after, limit } => {
            to_binary(&query_tokens_list(deps, start_after, limit)?)
        }
    }
}

fn query_tokens_list(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<TokenListResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    // get the entries that matches the range
    let entries: StdResult<Vec<_>> = entries()
        .idx
        .key
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let results = TokenListResponse {
        entries: entries?.into_iter().map(|l| l.1).collect(),
    };

    Ok(results)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = INIT_CONFIG.load(deps.storage)?;
    Ok(config)
}
