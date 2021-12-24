#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{read_config, store_config, Config};

use beth::converter::{ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};

use crate::math::{convert_to_anchor_decimals, convert_to_wormhole_decimals};
use crate::querier::query_decimals;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // cannot register the token at the inistantiation
    // because for the anchor token contract, converter needs to be minter.
    let conf = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        anchor_token_address: None,
        wormhole_token_address: None,
        anchor_decimals: 0,
        wormhole_decimals: 0,
    };

    store_config(deps.storage).save(&conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::RegisterTokens {
            anchor_token_address,
            wormhole_token_address,
        } => register_tokens(deps, info, anchor_token_address, wormhole_token_address),
    }
}

/// CW20 token receive handler.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender.clone();

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::ConvertWormholeToAnchor {}) => {
            // only wormhole beth token contract can execute this message
            let conf = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())?
                != conf.wormhole_token_address.unwrap()
            {
                return Err(StdError::generic_err("unauthorized"));
            }
            execute_convert_to_anchor(deps, env, info, cw20_msg.amount, cw20_msg.sender)
        }
        Ok(Cw20HookMsg::ConvertAnchorToWormhole {}) => {
            // only anchor beth token contract can execute this message
            let conf = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())?
                != conf.anchor_token_address.unwrap()
            {
                return Err(StdError::generic_err("unauthorized"));
            }
            execute_convert_to_wormhole(deps, env, info, cw20_msg.amount, cw20_msg.sender)
        }
        Err(err) => Err(err),
    }
}

pub fn register_tokens(
    deps: DepsMut,
    info: MessageInfo,
    anchor_token_address: String,
    wormhole_token_address: String,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // if the token contract is  already register we cannot change the address
    if config.anchor_token_address.is_none() {
        config.anchor_token_address = Some(deps.api.addr_canonicalize(&anchor_token_address)?);
        let anchor_decimals = query_decimals(deps.as_ref(), Addr::unchecked(anchor_token_address))?;
        config.anchor_decimals = anchor_decimals;
    }

    // if the token contract is  already register we cannot change the address
    if config.wormhole_token_address.is_none() {
        config.wormhole_token_address = Some(deps.api.addr_canonicalize(&wormhole_token_address)?);
        let wormhole_decimals =
            query_decimals(deps.as_ref(), Addr::unchecked(wormhole_token_address))?;
        config.wormhole_decimals = wormhole_decimals;
    }

    store_config(deps.storage).save(&config)?;

    Ok(Response::new().add_attributes(vec![("action", "register_token_contracts")]))
}

pub(crate) fn execute_convert_to_anchor(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    // should convert to anchor decimals
    let mint_amount =
        convert_to_anchor_decimals(amount, config.anchor_decimals, config.wormhole_decimals)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.anchor_token_address.unwrap())?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                amount: mint_amount,
            })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            ("action", "convert-to-anchor"),
            ("recipient", &sender),
            ("minted_amount", &mint_amount.to_string()),
        ]))
}

pub(crate) fn execute_convert_to_wormhole(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    // should convert to wormhole decimals
    let return_amount =
        convert_to_wormhole_decimals(amount, config.anchor_decimals, config.wormhole_decimals)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&config.wormhole_token_address.unwrap())?
                    .to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: sender.clone(),
                    amount: return_amount,
                })?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&config.anchor_token_address.unwrap())?
                    .to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            }),
        ])
        .add_attributes(vec![
            ("action", "convert-to-wormhole"),
            ("recipient", &sender),
            ("return_amount", &return_amount.to_string()),
            ("born_amount", &amount.to_string()),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    let anchor_token = if config.anchor_token_address.is_some() {
        Some(
            deps.api
                .addr_humanize(&config.anchor_token_address.unwrap())?
                .to_string(),
        )
    } else {
        None
    };
    let wormhole_token = if config.wormhole_token_address.is_some() {
        Some(
            deps.api
                .addr_humanize(&config.wormhole_token_address.unwrap())?
                .to_string(),
        )
    } else {
        None
    };
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        anchor_token_address: anchor_token,
        anchor_decimals: config.anchor_decimals,
        wormhole_token_address: wormhole_token,
        wormhole_decimals: config.wormhole_decimals,
    })
}
