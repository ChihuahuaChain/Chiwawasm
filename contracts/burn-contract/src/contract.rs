#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, INIT_CONFIG};

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Get the owner of the contract
    let owner = msg
        .owner
        .and_then(|addr_str| deps.api.addr_validate(addr_str.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };

    // save the owner to the INIT_CONFIG state
    INIT_CONFIG.save(deps.storage, &config)?;

    // return response
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BurnContractBalance {} => execute_burn_balance(deps, info, env),
        ExecuteMsg::TransferContractOwnership { new_owner } => {
            execute_transfer_owner(deps, info, new_owner)
        }
    }
}

fn execute_transfer_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    // Before we create a new entry, we check to see if the message sender is the owner of the contract
    let owner = INIT_CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate new owner
    let new_owner = deps.api.addr_validate(&new_owner)?;

    // Here we update the owner in the config
    let updated_config = INIT_CONFIG.update(deps.storage, |mut data| -> StdResult<_> {
        data.owner = new_owner;

        Ok(data)
    })?;

    Ok(Response::new()
        .add_attribute("method", "execute_transfer_owner")
        .add_attribute("new_owner", updated_config.owner))
}

fn execute_burn_balance(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    // Before we create a new entry, we check to see if the message sender is the owner of the contract
    let owner = INIT_CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // Get the contract balances
    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    // create a burn message
    let burn_msg = BankMsg::Burn { amount };

    // Then we add the message to the response
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    // Build response
    let res = Response::new()
        .add_attribute("method", "try_burn_balance")
        .add_messages(msgs);

    // return response
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Match and route the query message to the appropriate handler
    match msg {
        QueryMsg::QueryBalance {} => to_binary(&query_balance(deps, env)?),
    }
}

fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    // Get the contract balances
    let amount = deps
        .querier
        .query_all_balances(&env.contract.address)?
        .first()
        .unwrap()
        .clone();

    Ok(BalanceResponse {
        balance: amount.to_string(),
    })
}

#[cfg(test)]
mod tests;
