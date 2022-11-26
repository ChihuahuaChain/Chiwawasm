#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, BURN_READY_TIMESTAMP, INIT_CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:burn-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        daily_burn_amount: Uint128::from(msg.daily_burn_amount),
        burn_delay_seconds: msg.burn_delay_seconds,
        native_denom: msg.native_denom,
    };

    // save the owner to the INIT_CONFIG state
    INIT_CONFIG.save(deps.storage, &config)?;

    // Set the BURN_READY_TIMESTAMP to now
    let now = _env.block.time;
    BURN_READY_TIMESTAMP.save(deps.storage, &now)?;

    // return response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BurnDailyQuota {} => execute_burn_daily_quota(deps, env),
    }
}

fn execute_burn_daily_quota(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let burn_ready_time = BURN_READY_TIMESTAMP.load(deps.storage)?;
    let config = INIT_CONFIG.load(deps.storage)?;
    let now = env.block.time;

    if now < burn_ready_time {
        return Err(ContractError::DailyBurnNotReady {});
    }

    // update the next burn time
    let next_burn_time = now.plus_seconds(config.burn_delay_seconds);
    BURN_READY_TIMESTAMP.save(deps.storage, &next_burn_time)?;

    // To get the amount of coins to be burned
    // find the coin with non-zero balance that matches the denom
    let contract_balances = deps.querier.query_all_balances(&env.contract.address)?;
    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == config.native_denom && !coin.amount.is_zero());

    let coin = match coin {
        Some(coin) => match coin.amount > config.daily_burn_amount {
            true => Coin {
                amount: config.daily_burn_amount,
                denom: config.native_denom,
            },
            false => coin.clone(),
        },
        None => {
            return Err(ContractError::InsufficientContractBalance {});
        }
    };

    // we can now proceed to burning the coins
    // create a burn message
    let amount = [coin].to_vec();
    let burn_msg = BankMsg::Burn { amount };

    // Then we add the message to the response
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    // Build response
    let res = Response::new()
        .add_attribute("method", "execute_burn_daily_quota")
        .add_messages(msgs);

    // return response
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::SetMaxDailyBurn { amount } => sudo_set_max_daily_burn(deps, amount),
        SudoMsg::WithdrawFundsToCommunityPool { address } => {
            sudo_withdraw_funds_to_address(deps, env, address)
        }
    }
}

fn sudo_set_max_daily_burn(deps: DepsMut, amount: u128) -> Result<Response, ContractError> {
    // Here we update the owner in the config
    let updated_config = INIT_CONFIG.update(deps.storage, |mut data| -> StdResult<_> {
        data.daily_burn_amount = Uint128::from(amount);

        Ok(data)
    })?;

    Ok(Response::new()
        .add_attribute("method", "sudo_set_max_daily_burn")
        .add_attribute("daily_burn_amount", updated_config.daily_burn_amount))
}

fn sudo_withdraw_funds_to_address(
    deps: DepsMut,
    env: Env,
    address: String,
) -> Result<Response, ContractError> {
    let config = INIT_CONFIG.load(deps.storage)?;

    // we validate the address
    let address = deps.api.addr_validate(&address)?;

    // find the coin with non-zero balance that matches the denom
    let contract_balances = deps.querier.query_all_balances(&env.contract.address)?;
    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == config.native_denom && !coin.amount.is_zero());

    let coin = match coin {
        Some(coin) => coin.clone(),
        None => {
            return Err(ContractError::InsufficientContractBalance {});
        }
    };

    // we can now proceed to transfering the contract balance to the provided address
    // create a burn message
    let amount = [coin].to_vec();
    let send_msg = BankMsg::Send {
        amount,
        to_address: address.into_string(),
    };

    // Then we add the message to the response
    let msgs: Vec<CosmosMsg> = vec![send_msg.into()];

    // Build response
    let res = Response::new()
        .add_attribute("method", "sudo_withdraw_funds_to_address")
        .add_messages(msgs);

    // return response
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Match and route the query message to the appropriate handler
    match msg {
        QueryMsg::Balance {} => to_binary(&query_balance(deps, env)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    // get contract balances
    let contract_balances = deps.querier.query_all_balances(&env.contract.address)?;
    let denom = INIT_CONFIG.load(deps.storage)?.native_denom;

    let default = Coin {
        amount: Uint128::from(0u128),
        denom: denom.clone(),
    };

    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == denom)
        .unwrap_or(&default);

    Ok(BalanceResponse {
        amount: Coin {
            amount: coin.amount,
            denom: coin.denom.clone(),
        },
    })
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = INIT_CONFIG.load(deps.storage)?;
    Ok(config)
}
