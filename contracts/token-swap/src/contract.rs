use std::str::FromStr;

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw20::{Expiration, MinterResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
use crate::state::{Config, Token, BASE_TOKEN, CONFIG, LP_TOKEN, QUOTE_TOKEN};

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

    // check to make sure base_denom != quote_denom
    if msg.base_denom.clone().eq(&msg.quote_denom.clone()) {
        return Err(ContractError::IdenticalDenomNotAllowedInPair {});
    }

    // Make sure the base_denom == native_denom
    if msg.native_denom.clone().ne(&msg.base_denom.clone()) {
        return Err(ContractError::NativeTokenNotProvidedInPair {});
    }

    // check that the swap_rate is between 0.1% and 1.0%
    if msg.swap_rate.clone() < Decimal::from_str("0.1").unwrap()
        || msg.swap_rate.clone() > Decimal::one()
    {
        return Err(ContractError::InvalidSwapRate {});
    }

    // Save the swap rate
    CONFIG.save(
        deps.storage,
        &Config {
            swap_rate: msg.swap_rate,
        },
    )?;

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
            token1_amount,
            min_liquidity,
            max_token2,
            expiration,
        } => execute_add_liquidity(
            deps,
            &info,
            env,
            min_liquidity,
            token1_amount,
            max_token2,
            expiration,
        ),
        ExecuteMsg::RemoveLiquidity {
            amount,
            min_token1,
            min_token2,
            expiration,
        } => execute_remove_liquidity(deps, info, env, amount, min_token1, min_token2, expiration),
        ExecuteMsg::Swap {
            input_token,
            input_amount,
            min_output,
            expiration,
            ..
        } => execute_swap(
            deps,
            &info,
            input_amount,
            env,
            input_token,
            &info.sender,
            min_output,
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
            recipient,
            min_token,
            expiration,
        } => execute_swap(
            deps,
            &info,
            input_amount,
            env,
            input_token,
            &recipient,
            min_token,
            expiration,
        ),
    }
}

// todo
pub fn execute_add_liquidity(
    deps: DepsMut,
    info: &MessageInfo,
    env: Env,
    min_liquidity: Uint128,
    token1_amount: Uint128,
    max_token2: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

// todo
pub fn execute_remove_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    min_token1: Uint128,
    min_token2: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

// todo
pub fn execute_swap(
    deps: DepsMut,
    info: &MessageInfo,
    input_amount: Uint128,
    _env: Env,
    input_token_enum: TokenSelect,
    recipient: &Addr,
    min_token: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

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
    let config = CONFIG.load(deps.storage)?;
    let lp_token_address = LP_TOKEN.load(deps.storage)?;

    Ok(InfoResponse {
        base_reserve: base.reserve,
        base_denom: base.denom,
        quote_reserve: quote.reserve,
        quote_denom: quote.denom,
        swap_rate: config.swap_rate,
        lp_token_supply: get_lp_token_supply(deps, &lp_token_address)?,
        lp_token_address: lp_token_address,
    })
}

fn get_lp_token_supply(deps: Deps, lp_token_addr: &Addr) -> StdResult<Uint128> {
    let resp: cw20::TokenInfoResponse = deps
        .querier
        .query_wasm_smart(lp_token_addr, &cw20_base::msg::QueryMsg::TokenInfo {})?;
    Ok(resp.total_supply)
}
