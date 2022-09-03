use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo, TokenListResponse};
use crate::state::{
    entries, Config, Entry, TempEntry, DEFAULT_LIMIT, ENTRY_SEQ, INIT_CONFIG, MAX_LIMIT,
    TEMP_ENTRY_STATE,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Reply, ReplyOn,
    Response, StdError, StdResult, Storage, SubMsg, WasmMsg,
};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_REPLY_ID: u64 = 1u64;

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
        token_creation_fee: msg.token_creation_fee,
        token_code_id: msg.token_code_id,
    };

    // save INIT_CONFIG state
    INIT_CONFIG.save(deps.storage, &config)?;

    // save the entry sequence to storage starting from 0
    ENTRY_SEQ.save(deps.storage, &0u64)?;

    // return response
    Ok(Response::new().add_attribute("method", "instantiate"))
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

    // Check if a token with the same symbol already exists
    let entry = entries().may_load(deps.storage, &token_info.symbol.to_lowercase())?;
    if let Some(_) = entry {
        return Err(ContractError::TokenWithSymbolAlreadyExists {
            symbol: token_info.symbol,
        });
    }

    // Check the sub_index if token with the same name already exists
    /*let entry = entries()
        .idx
        .name
        .item(deps.storage, token_info.name.to_lowercase())?;
    if let Some(_) = entry {
        return Err(ContractError::TokenWithNameAlreadyExists {
            name: token_info.name,
        });
    }*/

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

    // Save the TempEntry to state
    let entry = TempEntry {
        name: token_info.name.to_lowercase(),
        symbol: token_info.symbol.to_lowercase(),
        logo: token_info.marketing.logo.clone(),
    };
    TEMP_ENTRY_STATE.save(deps.storage, &entry)?;

    // Add message to burn the token_creation_fee
    let amount = vec![fee];
    let burn_msg = BankMsg::Burn { amount };
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    // Add wasm msg to create new cw20 token instance from config.token_code_id
    let instantiate_message = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: config.token_code_id,
        msg: to_binary(&token_info)?,
        funds: vec![],
        label: token_info.name.to_string(),
    };
    let sub_msg = SubMsg {
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
        msg: instantiate_message.into(),
    };

    // Build response
    let res = Response::new()
        .add_attribute("method", "execute_create_new_token")
        .add_messages(msgs)
        .add_submessage(sub_msg);

    // return response
    Ok(res)
}

/**
 * Handle reply for execute_create_new_token
 * Load the TEMP_ENTRY_STATE and use the data to create a new Entry that includes
 * the newly created contract_address and latest ENTRY_SEQ id,
 *
 * Save it to the store under (name, symbol): Entry of fn entries();
 * @return the token_contract_addr as an attribute on success
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_REPLY_ID => handle_instantiate_reply(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    // Get data from reply msg
    // See: https://github.com/CosmWasm/cw-plus/blob/main/packages/utils/src/parse_reply.rs
    let res = parse_reply_instantiate_data(msg);
    let data = match res {
        Ok(d) => d,
        Err(_) => {
            return Err(StdError::generic_err("Error parsing data"));
        }
    };

    // Get temp_entry and next id
    let temp_entry = TEMP_ENTRY_STATE.load(deps.storage)?;
    let id = next_entry_seq(deps.storage)?;

    // Save the actual Entry
    let entry = Entry {
        id,
        contract_addr: deps.api.addr_validate(&data.contract_address)?,
        name: temp_entry.name,
        symbol: temp_entry.symbol,
        logo: temp_entry.logo,
    };
    entries().save(deps.storage, entry.symbol.as_str(), &entry)?;

    Ok(Response::new().add_attribute("token_contract_addr", data.contract_address))
}

pub fn next_entry_seq(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = ENTRY_SEQ.may_load(store)?.unwrap_or_default() + 1;
    ENTRY_SEQ.save(store, &id)?;
    Ok(id)
}

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
        .id
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let results = TokenListResponse {
        entries: entries?.into_iter().map(|l| l.1).collect(),
    };

    Ok(results)
}

// ? nice to have: query token by name or symbol

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = INIT_CONFIG.load(deps.storage)?;
    Ok(config)
}
