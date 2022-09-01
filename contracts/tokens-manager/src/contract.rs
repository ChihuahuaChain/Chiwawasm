#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};

use crate::state::{Config, INIT_CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, ContractError> {
    //Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // save the owner to the INIT_CONFIG state
    let config = Config {
        owner: info.sender.clone(),
    };
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
    // Check if the caller is the owner of the contract
    let owner = INIT_CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // also check if the token_info is valid
    token_info.validate()?;

    // todo add logic to create new token

    Ok(Response::new().add_attribute("method", "execute_create_new_token"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Match and route the query message to the appropriate handler
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = INIT_CONFIG.load(deps.storage)?;
    Ok(config)
}
