use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdResult, Uint128,
};

// contract info
pub const CONTRACT_NAME: &str = "tx-burn-contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Save contract state
    CONFIG.save(
        _deps.storage,
        &Config {
            admin: _info.sender,
            max_balance_to_burn: _msg.max_balance_to_burn,
            multiplier: _msg.multiplier,
            balance_burned_already: Uint128::zero(),
        },
    )?;

    // response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::BurnTokens { amount } => execute_burn_tokens(_deps, _env, &_info, amount),
        ExecuteMsg::UpdatePreferences {
            max_burn_amount,
            multiplier,
        } => execute_update_preferences(_deps, _env, &_info, max_burn_amount, multiplier),
        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            execute_withdraw_balance(_deps, _env, &_info, to_address, funds)
        }
    }
}

fn verify_caller_is_admin(info: &MessageInfo, deps: &DepsMut) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_admin = info.sender.eq(&config.admin);

    if !is_admin {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn get_available_balace(
    env: &Env,
    deps: &DepsMut,
    denom_str: String,
) -> Result<Coin, ContractError> {
    // find the coin with non-zero balance that matches the denom
    let contract_balances = deps
        .querier
        .query_all_balances(env.contract.address.clone())?;

    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == denom_str);

    Ok(match coin {
        Some(coin) => coin.clone(),
        None => Coin {
            amount: Uint128::zero(),
            denom: denom_str,
        },
    })
}

fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, amount: Uint128) -> CosmosMsg {
    let transfer_bank_msg = BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount,
        }],
    };

    let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();
    transfer_bank_cosmos_msg
}

pub fn execute_burn_tokens(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_update_preferences(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    max_burn_amount: Option<Uint128>,
    multiplier: Option<u8>,
) -> Result<Response, ContractError> {
    verify_caller_is_admin(&info, &deps)?;

    // Update when max_burn_amount > config.balance_burned_already
    if let Some(val) = max_burn_amount {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            if val > config.balance_burned_already {
                config.max_balance_to_burn = val;
            }

            Ok(config)
        })?;
    }

    // Update multiplier
    if let Some(val) = multiplier {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            config.multiplier = val;
            Ok(config)
        })?;
    }

    // Respond
    Ok(Response::new().add_attributes(vec![attr("method", "update_preferences")]))
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    verify_caller_is_admin(&info, &deps)?;

    // Check if the contract balance is >= the requested amount to withdraw
    let available_balance = get_available_balace(&env, &deps, funds.denom.clone())?;
    if available_balance.amount < funds.amount {
        return Err(ContractError::InsufficientBalance {
            available: available_balance,
            required: funds,
        });
    }

    // Get the recipient to send funds to
    let recipient: Addr;
    if let Some(val) = to_address {
        recipient = deps.api.addr_validate(&val)?;
    } else {
        let config = CONFIG.load(deps.storage)?;
        recipient = config.admin;
    }

    // construct sdk msg to transfer funds to recipient
    let msg = get_bank_transfer_to_msg(&recipient, &funds.denom, funds.amount);

    Ok(Response::new().add_message(msg).add_attributes(vec![
        attr("method", "withdraw_balance"),
        attr("recipient", recipient.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(_deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(config)
}
