use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, BlockInfo, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw20::{Cw20ExecuteMsg, Denom, Expiration, MinterResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
use crate::state::{SwapPrice, Token, TokenAmount, BASE_TOKEN, LP_TOKEN, QUOTE_TOKEN};

// Version info for migration info
pub const CONTRACT_NAME: &str = "huahuaswap";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check that base denom is of Native token type
    match msg.base_denom {
        Denom::Native(_) => {}
        _ => {
            return Err(ContractError::InvalidBaseDenom {});
        }
    }

    // Make sure the base_denom == native_denom
    if msg.native_denom.clone().ne(&msg.base_denom.clone()) {
        return Err(ContractError::NativeTokenNotProvidedInPair {});
    }

    // Verify that quote denom is of cw20 token type
    // todo we can extedn the Denom to support IBC denom type as well
    match msg.quote_denom {
        Denom::Cw20(_) => {}
        _ => {
            return Err(ContractError::InvalidQuoteDenom {});
        }
    }

    // Save base token
    BASE_TOKEN.save(
        deps.storage,
        &Token {
            reserve: Uint128::zero(),
            denom: msg.base_denom.clone(),
        },
    )?;

    // Save quote token
    QUOTE_TOKEN.save(
        deps.storage,
        &Token {
            denom: msg.quote_denom.clone(),
            reserve: Uint128::zero(),
        },
    )?;

    // Add submessage for creating the LP token for this pool
    let sub_msg = SubMsg {
        gas_limit: None,
        id: INSTANTIATE_LP_TOKEN_REPLY_ID,
        reply_on: ReplyOn::Success,
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.lp_token_code_id,
            msg: to_binary(&cw20_base::msg::InstantiateMsg {
                name: "HuahuaSwap LP Token".into(),
                symbol: "hhslpt".into(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.into(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: format!("hhslp_{:?}_{:?}", msg.base_denom, msg.quote_denom),
        }
        .into(),
    };

    // Build response
    let res = Response::new()
        .add_attribute("method", "instantiate")
        .add_submessage(sub_msg);

    // return response
    Ok(res)
}

/**
 * Handle reply for contract instantiation
 * Get the contract address and save as P_TOKEN
 *
 * @return the token_contract_addr as an attribute on success
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_LP_TOKEN_REPLY_ID => handle_instantiate_reply(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let res = parse_reply_instantiate_data(msg);
    let data = match res {
        Ok(d) => d,
        Err(_) => {
            return Err(StdError::generic_err("Error parsing data"));
        }
    };

    // Validate contract address
    let cw20_addr = deps.api.addr_validate(&data.contract_address)?;

    // Save gov token
    LP_TOKEN.save(deps.storage, &cw20_addr)?;

    Ok(Response::new().add_attribute("token_contract_addr", data.contract_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddLiquidity {
            base_token_amount,
            max_quote_token_amount,
            expiration,
        } => execute_add_liquidity(
            deps,
            &info,
            env,
            base_token_amount,
            max_quote_token_amount,
            expiration,
        ),
        ExecuteMsg::RemoveLiquidity {
            amount,
            min_base_token_output,
            min_quote_token_output,
            expiration,
        } => execute_remove_liquidity(
            deps,
            info,
            env,
            amount,
            min_base_token_output,
            min_quote_token_output,
            expiration,
        ),
        ExecuteMsg::Swap {
            input_token,
            input_amount,
            output_amount,
            expiration,
        } => execute_swap(
            env,
            deps,
            &info,
            input_amount,
            input_token,
            output_amount,
            &info.sender,
            expiration,
        ),
        ExecuteMsg::PassThroughSwap {
            output_amm_address,
            input_token,
            input_token_amount,
            output_min_token,
            expiration,
        } => execute_pass_through_swap(
            deps,
            info,
            env,
            output_amm_address,
            input_token,
            input_token_amount,
            output_min_token,
            expiration,
        ),
        ExecuteMsg::SwapAndSendTo {
            input_token,
            input_amount,
            output_amount,
            recipient,
            expiration,
        } => execute_swap(
            env,
            deps,
            &info,
            input_amount,
            input_token,
            output_amount,
            &recipient,
            expiration,
        ),
    }
}

fn check_expiration(
    expiration: &Option<Expiration>,
    block: &BlockInfo,
) -> Result<(), ContractError> {
    if let Some(e) = expiration {
        if e.is_expired(block) {
            return Err(ContractError::MsgExpirationError {});
        }
    }

    Ok(())
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

fn validate_base_amount(
    coins: &[Coin],
    given_amount: Uint128,
    denom_str: &str,
) -> Result<(), ContractError> {
    let actual = get_amount_for_denom(coins, denom_str);

    if actual.amount != given_amount {
        return Err(ContractError::InsufficientFunds {});
    }

    Ok(())
}

fn get_lp_token_supply(deps: Deps, lp_token_addr: &Addr) -> StdResult<Uint128> {
    let resp: cw20::TokenInfoResponse = deps
        .querier
        .query_wasm_smart(lp_token_addr, &cw20_base::msg::QueryMsg::TokenInfo {})?;
    Ok(resp.total_supply)
}

fn get_token_balance(deps: Deps, contract: &Addr, addr: &Addr) -> StdResult<Uint128> {
    let resp: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        contract,
        &cw20_base::msg::QueryMsg::Balance {
            address: addr.to_string(),
        },
    )?;
    Ok(resp.balance)
}

pub fn get_lp_token_amount_to_mint(
    base_token_amount: Uint128,
    liquidity_supply: Uint128,
    base_reserve: Uint128,
) -> Result<Uint128, ContractError> {
    if liquidity_supply == Uint128::zero() {
        Ok(base_token_amount)
    } else {
        Ok(base_token_amount
            .checked_mul(liquidity_supply)
            .map_err(StdError::overflow)?
            .checked_div(base_reserve)
            .map_err(StdError::divide_by_zero)?)
    }
}

pub fn get_required_quote_token_amount(
    base_token_amount: Uint128,
    liquidity_supply: Uint128,
    quote_reserve: Uint128,
    base_reserve: Uint128,
) -> Result<Uint128, StdError> {
    if liquidity_supply == Uint128::zero() {
        Ok(base_token_amount)
    } else {
        Ok(base_token_amount
            .checked_mul(quote_reserve)
            .map_err(StdError::overflow)?
            .checked_div(base_reserve)
            .map_err(StdError::divide_by_zero)?)
    }
}

fn get_cw20_transfer_from_msg(
    owner: &Addr,
    recipient: &Addr,
    token_addr: &Addr,
    token_amount: Uint128,
) -> StdResult<CosmosMsg> {
    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::TransferFrom {
        owner: owner.into(),
        recipient: recipient.into(),
        amount: token_amount,
    };
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: token_addr.into(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    };
    let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();
    Ok(cw20_transfer_cosmos_msg)
}

fn mint_lp_tokens(
    recipient: &Addr,
    liquidity_amount: Uint128,
    lp_token_address: &Addr,
) -> StdResult<CosmosMsg> {
    let mint_msg = cw20_base::msg::ExecuteMsg::Mint {
        recipient: recipient.into(),
        amount: liquidity_amount,
    };
    Ok(WasmMsg::Execute {
        contract_addr: lp_token_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    }
    .into())
}

fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, native_amount: Uint128) -> CosmosMsg {
    let transfer_bank_msg = BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.to_string(),
            amount: native_amount,
        }],
    };

    let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();
    transfer_bank_cosmos_msg
}

fn get_cw20_transfer_to_msg(
    recipient: &Addr,
    token_addr: &Addr,
    token_amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(WasmMsg::Execute {
        contract_addr: token_addr.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount: token_amount,
        })?,
        funds: vec![],
    }
    .into())
}

fn get_burn_msg(contract: &Addr, owner: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
    let msg = cw20_base::msg::ExecuteMsg::BurnFrom {
        owner: owner.to_string(),
        amount,
    };
    Ok(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    }
    .into())
}

pub fn execute_add_liquidity(
    deps: DepsMut,
    info: &MessageInfo,
    env: Env,
    base_token_amount: Uint128,
    max_quote_token_amount: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;

    // Check that non zero amounts are passed for both tokens
    if base_token_amount.is_zero() || max_quote_token_amount.is_zero() {
        return Err(ContractError::NonZeroInputAmountExpected {});
    }

    // load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_addr = LP_TOKEN.load(deps.storage)?;

    // Validate the input for the base_token_amount to know if the user
    // sent the exact amount and denom in the contract call
    if let Denom::Native(denom) = base.denom {
        validate_base_amount(&info.funds, base_token_amount, &denom)?;
    }

    // Calculate how much lp tokens to mint
    let lp_token_supply = get_lp_token_supply(deps.as_ref(), &lp_token_addr)?;
    let liquidity_amount =
        get_lp_token_amount_to_mint(base_token_amount, lp_token_supply, base.reserve)?;

    // Calculate the required_quote_token_amount
    let required_quote_token_amount = get_required_quote_token_amount(
        base_token_amount,
        lp_token_supply,
        quote.reserve,
        base.reserve,
    )?;

    // Validate that max_quote_token_amount <= required_quote_token_amount
    if required_quote_token_amount > max_quote_token_amount {
        return Err(ContractError::MaxQuoteTokenAmountExceeded {
            max_quote_token_amount,
            required_quote_token_amount,
        });
    }

    // Generate CW20 transfer message to transfer required_quote_token_amount
    // from caller address to contract address
    let mut transfer_msg = vec![];
    if let Denom::Cw20(addr) = quote.denom.clone() {
        transfer_msg.push(get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &addr,
            required_quote_token_amount,
        )?);
    }

    // Update token reserves
    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
        base.reserve += base_token_amount;
        Ok(base)
    })?;
    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
        quote.reserve += required_quote_token_amount;
        Ok(quote)
    })?;

    // Mint LP tokens
    let mint_msg = mint_lp_tokens(&info.sender, liquidity_amount, &lp_token_addr)?;

    // respond
    Ok(Response::new()
        .add_messages(transfer_msg)
        .add_message(mint_msg)
        .add_attributes(vec![
            attr("base_token_amount", base_token_amount),
            attr("required_quote_token_amount", required_quote_token_amount),
            attr("liquidity_received", liquidity_amount),
        ]))
}

pub fn execute_remove_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    lp_amount: Uint128,
    min_base_token_output: Uint128,
    min_quote_token_output: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;

    // load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_addr = LP_TOKEN.load(deps.storage)?;
    let lp_token_supply = get_lp_token_supply(deps.as_ref(), &lp_token_addr)?;
    let user_lp_balance = get_token_balance(deps.as_ref(), &lp_token_addr, &info.sender)?;

    // Check if lp amount to withdraw is valid
    if lp_amount > user_lp_balance {
        return Err(ContractError::InsufficientLiquidityError {
            requested: lp_amount,
            available: user_lp_balance,
        });
    }

    // Calculate the base token amount to withdraw from the pool
    let base_amount_to_output = lp_amount
        .checked_mul(base.reserve)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;
    if base_amount_to_output < min_base_token_output {
        return Err(ContractError::MinBaseTokenOutputError {
            requested: min_base_token_output,
            available: base_amount_to_output,
        });
    }

    // Calculate the quote token amount to withdraw from the pool
    let quote_amount_to_output = lp_amount
        .checked_mul(quote.reserve)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;

    if quote_amount_to_output < min_quote_token_output {
        return Err(ContractError::MinQuoteTokenOutputError {
            requested: min_quote_token_output,
            available: quote_amount_to_output,
        });
    }

    // Update token reserves
    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
        base.reserve = base
            .reserve
            .checked_sub(base_amount_to_output)
            .map_err(StdError::overflow)?;
        Ok(base)
    })?;
    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
        quote.reserve = quote
            .reserve
            .checked_sub(quote_amount_to_output)
            .map_err(StdError::overflow)?;
        Ok(quote)
    })?;

    // Construct the messages to send the output tokens to info.sender
    let base_token_transfer_msg = if let Denom::Native(denom) = base.denom {
        get_bank_transfer_to_msg(&info.sender, &denom, base_amount_to_output)
    } else {
        // This is to satisfy the compulsory else block, it will never be called
        return Err(ContractError::NoneError {});
    };
    let quote_token_transfer_msg = if let Denom::Cw20(addr) = quote.denom {
        get_cw20_transfer_to_msg(&info.sender, &addr, quote_amount_to_output)?
    } else {
        // This is to satisfy the compulsory else block, it will never be called
        return Err(ContractError::NoneError {});
    };

    // Construct message to burn lp_amount
    let lp_token_burn_msg = get_burn_msg(&lp_token_addr, &info.sender, lp_amount)?;

    // respond
    Ok(Response::new()
        .add_messages(vec![
            base_token_transfer_msg,
            quote_token_transfer_msg,
            lp_token_burn_msg,
        ])
        .add_attributes(vec![
            attr("liquidity_burned", lp_amount),
            attr("base_token_returned", base_amount_to_output),
            attr("quote_token_returned", quote_amount_to_output),
        ]))
}

/*
 * When swapping from base token to quote token, we use fn exactInputVariableOutput {}
 * Where the input_amount is the exact amount of base tokens to be swapped for a variable amount
 * of the quote tokens such that calculated_quote_output >= output_amount
 * Here, output_amount represents min_quote_output_amount.
 *
 *
 * When swapping from quote token to base token, we use fn exactOutputVariableInput {}
 * Where the input_amount is the max limit of quote token to be inputed in exchange for an exact amount
 * of base token such that calculated_quote_input <= input_amount
 * Here, input_amount represents the max_quote_input_amount.
 *
 *
 * What this means is that the swap_fee is always charged to the quote token.
 */
pub fn execute_swap(
    _env: Env,
    deps: DepsMut,
    info: &MessageInfo,
    input_amount: Uint128,
    input_token: TokenSelect,
    output_amount: Uint128,
    recipient: &Addr,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &_env.block)?;

    // here we load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;

    // Here we get the swap_prices which is the amount of input and output tokens required
    let swap_price = match input_token {
        TokenSelect::Base => exact_input_variable_output(
            input_amount,
            output_amount,
            base.reserve,
            quote.reserve,
            base.denom.clone(),
            quote.denom.clone(),
        )?,

        TokenSelect::Quote => exact_output_variable_input(
            output_amount,
            input_amount,
            base.reserve,
            quote.reserve,
            base.denom.clone(),
            quote.denom.clone(),
        )?,
    };

    // Create SDK messages holder
    let mut messages: Vec<CosmosMsg> = vec![];

    // Update reserves and sdk messages for the input token
    match swap_price.input.denom.clone() {
        Denom::Native(denom) => {
            validate_base_amount(&info.funds, swap_price.input.amount, &denom)?;

            BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
                base.reserve += swap_price.input.amount;
                Ok(base)
            })?;
        }
        Denom::Cw20(addr) => {
            messages.push(get_cw20_transfer_from_msg(
                &info.sender,
                &_env.contract.address,
                &addr,
                swap_price.input.amount,
            )?);

            QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                quote.reserve += swap_price.input.amount;
                Ok(quote)
            })?;
        }
    }

    // Update reserves and sdk messages for the output token
    match swap_price.output.denom.clone() {
        Denom::Native(denom) => {
            messages.push(get_bank_transfer_to_msg(
                recipient,
                &denom,
                swap_price.output.amount,
            ));

            BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
                base.reserve -= swap_price.output.amount;
                Ok(base)
            })?;
        }
        Denom::Cw20(addr) => {
            messages.push(get_cw20_transfer_to_msg(
                recipient,
                &addr,
                swap_price.output.amount,
            )?);

            QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                quote.reserve -= swap_price.output.amount;
                Ok(quote)
            })?;
        }
    }

    // Respond
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("input_amount", swap_price.input.amount),
        attr("input_denom", format!("{:?}", swap_price.input.denom)),
        attr("output_amount", swap_price.output.amount),
        attr("output_denom", format!("{:?}", swap_price.output.denom)),
    ]))
}

/**
 * To output q and input b, we use
 * (B + b) * (Q - q) = k, where k = B * Q
 *
 * Differentiate for variable output q
 *
 * (Q - q) = k / (B + b)
 * q = Q - (BQ / (B + b))
 * q * (B + b)  =  Q *  (B + b) - BQ
 * q * (B + b) = QB + Qb - BQ
 * q = Qb / (B + b)
 */
pub fn exact_input_variable_output(
    exact_input_amount: Uint128,
    min_output_amount: Uint128,
    base_reserve: Uint128,
    quote_reserve: Uint128,
    base_denom: Denom,
    quote_denom: Denom,
) -> Result<SwapPrice, ContractError> {
    let numerator = quote_reserve
        .checked_mul(exact_input_amount)
        .map_err(StdError::overflow)?;

    let denominator = base_reserve
        .checked_add(exact_input_amount)
        .map_err(StdError::overflow)?;

    let calculated_quote_output = numerator
        .checked_div(denominator)
        .map_err(StdError::divide_by_zero)?;

    // Deduct swap_fee from the calculated_quote_output
    let swap_fee = get_swap_fee(calculated_quote_output)?;
    let calculated_quote_output = calculated_quote_output
        .checked_sub(swap_fee)
        .map_err(StdError::overflow)?;

    // make sure calculated_quote_output >= min_output_amount
    if calculated_quote_output < min_output_amount {
        return Err(ContractError::SwapMinError {
            min: min_output_amount,
            available: calculated_quote_output,
        });
    }

    Ok(SwapPrice {
        input: TokenAmount {
            amount: exact_input_amount,
            denom: base_denom,
        },
        output: TokenAmount {
            amount: calculated_quote_output,
            denom: quote_denom,
        },
    })
}

// Here we hardcode the swap fees as 3/1000 or 0.3% of amount
fn get_swap_fee(amount: Uint128) -> StdResult<Uint128> {
    amount
        .checked_mul(Uint128::from(3u128))
        .map_err(StdError::overflow)?
        .checked_div(Uint128::from(1000u128))
        .map_err(StdError::divide_by_zero)
}

/**
 * To output b and input q, we use
 * (B - b) * (Q + q) = k, where k = B * Q
 *
 * Differentiate for variable input q
 *
 * (Q + q) = k / (B - b)
 * q = -Q + ( k / (B - b))
 * q * (B - b) = -Q *  (B - b) + BQ
 * q * (B - b) = -QB + Qb + BQ
 * q = Qb / (B - b)
 */
pub fn exact_output_variable_input(
    exact_output_amount: Uint128,
    max_input_amount: Uint128,
    base_reserve: Uint128,
    quote_reserve: Uint128,
    base_denom: Denom,
    quote_denom: Denom,
) -> Result<SwapPrice, ContractError> {
    let numerator = quote_reserve
        .checked_mul(exact_output_amount)
        .map_err(StdError::overflow)?;

    let denominator = base_reserve
        .checked_sub(exact_output_amount)
        .map_err(StdError::overflow)?;

    let calculated_quote_input = numerator
        .checked_div(denominator)
        .map_err(StdError::divide_by_zero)?;

    // Add swap_fee to the calculated_quote_input
    let swap_fee = get_swap_fee(calculated_quote_input)?;
    let calculated_quote_input = calculated_quote_input
        .checked_add(swap_fee)
        .map_err(StdError::overflow)?;

    // make sure calculated_quote_input <= max_input_amount
    if calculated_quote_input > max_input_amount {
        return Err(ContractError::SwapMaxError {
            max: max_input_amount,
            required: calculated_quote_input,
        });
    }

    Ok(SwapPrice {
        input: TokenAmount {
            amount: calculated_quote_input,
            denom: quote_denom,
        },
        output: TokenAmount {
            amount: exact_output_amount,
            denom: base_denom,
        },
    })
}

// todo
pub fn execute_pass_through_swap(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    output_amm_address: Addr,
    input_token_enum: TokenSelect,
    input_token_amount: Uint128,
    output_min_token: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => {
            to_binary(&cw20_base::contract::query_balance(deps, address)?)
        }
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(deps: Deps) -> StdResult<InfoResponse> {
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_address = LP_TOKEN.load(deps.storage)?;

    Ok(InfoResponse {
        base_reserve: base.reserve,
        base_denom: base.denom,
        quote_reserve: quote.reserve,
        quote_denom: quote.denom,
        lp_token_supply: get_lp_token_supply(deps, &lp_token_address)?,
        lp_token_address: lp_token_address,
    })
}
