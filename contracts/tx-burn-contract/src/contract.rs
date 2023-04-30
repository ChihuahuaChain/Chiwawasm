use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128,
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
            max_extra_balance_to_burn_per_tx: _msg.max_extra_balance_to_burn_per_tx,
            multiplier: _msg.multiplier,
            total_amount_burned: Uint128::zero(),
            total_tx_burned: 0u64,
            total_balance_burned: Uint128::zero(),
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
            max_extra_burn_amount_per_tx,
            multiplier,
        } => execute_update_preferences(
            _deps,
            _env,
            &_info,
            max_extra_burn_amount_per_tx,
            multiplier,
        ),
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

fn get_amount_for_denom(coins: &[Coin], denom: &str) -> Coin {
    let amount: Uint128 = coins
        .iter()
        .filter(|c| c.denom == denom)
        .map(|c| c.amount)
        .sum();

    Coin {
        amount,
        denom: denom.to_string(),
    }
}

fn validate_exact_native_amount(
    deps: &DepsMut,
    coins: &[Coin],
    given_amount: Uint128,
) -> Result<(), ContractError> {
    let denom_str = deps.querier.query_bonded_denom()?;
    let actual = get_amount_for_denom(coins, denom_str.as_str());

    if actual.amount != given_amount {
        return Err(ContractError::IncorrectAmountProvided {
            provided: actual.amount,
            required: given_amount,
        });
    }

    Ok(())
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
    // make sure the caller sends the correct amount to the contract
    validate_exact_native_amount(&deps, &info.funds, amount)?;

    // Calculate extra amount to burn
    let config = CONFIG.load(deps.storage)?;
    let extra_amount_to_burn = amount
        .checked_mul(Uint128::from(config.multiplier))
        .map_err(StdError::overflow)?;

    // Calculate expected total_amount_to_burn
    let mut total_amount_to_burn = if extra_amount_to_burn > config.max_extra_balance_to_burn_per_tx
    {
        amount + config.max_extra_balance_to_burn_per_tx
    } else {
        amount + extra_amount_to_burn
    };

    // When the contract balance is less than total_amount_to_burn,
    // total_amount_to_burn is set to the contract balance
    let denom_str = deps.querier.query_bonded_denom()?;
    let available_balance = get_available_balace(&env, &deps, denom_str.clone())?;
    if total_amount_to_burn > available_balance.amount {
        total_amount_to_burn = available_balance.amount;
    }

    // Update config state
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        // Update total_amount_burned and total_tx_burned
        data.total_amount_burned += total_amount_to_burn;
        data.total_tx_burned += 1;

        // Update balance burned
        let balance_burned = total_amount_to_burn - amount;
        data.total_balance_burned += balance_burned;
        Ok(data)
    })?;

    // Proceed to burning total_amount_to_burn
    let burn_msg = BankMsg::Burn {
        amount: vec![Coin {
            amount: total_amount_to_burn,
            denom: denom_str,
        }],
    };

    // Build response
    Ok(Response::new().add_message(burn_msg).add_attributes(vec![
        attr("method", "burn_tokens"),
        attr("total_amount_to_burn", total_amount_to_burn.to_string()),
    ]))
}

pub fn execute_update_preferences(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    max_extra_balance_to_burn_per_tx: Option<Uint128>,
    multiplier: Option<u8>,
) -> Result<Response, ContractError> {
    verify_caller_is_admin(&info, &deps)?;

    // Update max_extra_balance_to_burn_per_tx when max_burn_amount > config.balance_burned_already
    if let Some(val) = max_extra_balance_to_burn_per_tx {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            config.max_extra_balance_to_burn_per_tx = val;
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
