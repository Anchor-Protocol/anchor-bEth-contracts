#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{read_config, store_config, Config};

use beth::swap::{ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    from_binary, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        anchor_beth_token: deps.api.addr_canonicalize(&msg.anchor_beth_token)?,
        wormhole_beth_token: deps.api.addr_canonicalize(&msg.wormhole_beth_token)?,
    };

    store_config(deps.storage).save(&conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            anchor_beth_token,
            wormhole_beth_token,
        } => update_config(deps, info, owner, anchor_beth_token, wormhole_beth_token),
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
        Ok(Cw20HookMsg::Bond {}) => {
            // only wormhole beth token contract can execute this message
            let conf = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != conf.wormhole_beth_token {
                return Err(StdError::generic_err("unauthorized"));
            }
            execute_bond(deps, env, info, cw20_msg.amount, cw20_msg.sender)
        }
        Ok(Cw20HookMsg::Unbond {}) => {
            // only anchor beth token contract can execute this message
            let conf = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != conf.anchor_beth_token {
                return Err(StdError::generic_err("unauthorized"));
            }
            execute_unbond(deps, env, info, cw20_msg.amount, cw20_msg.sender)
        }
        Err(err) => Err(err),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    anchor_beth_token: Option<String>,
    wormhole_beth_token: Option<String>,
) -> StdResult<Response> {
    let api = deps.api;
    store_config(deps.storage).update(|mut config| {
        if config.owner != api.addr_canonicalize(info.sender.as_str())? {
            return Err(StdError::generic_err("unauthorized"));
        }

        if let Some(owner) = owner {
            config.owner = api.addr_canonicalize(&owner)?;
        }

        if let Some(anchor_beth_token) = anchor_beth_token {
            config.anchor_beth_token = api.addr_canonicalize(&anchor_beth_token)?;
        }

        if let Some(wormhole_beth_token) = wormhole_beth_token {
            config.wormhole_beth_token = api.addr_canonicalize(&wormhole_beth_token)?;
        }
        Ok(config)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub(crate) fn execute_bond(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.anchor_beth_token)?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                amount,
            })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            ("action", "bond"),
            ("recipient", &sender),
            ("minted_amount", &amount.to_string()),
        ]))
}

pub(crate) fn execute_unbond(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&config.wormhole_beth_token)?
                    .to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: sender.clone(),
                    amount,
                })?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&config.anchor_beth_token)?
                    .to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            }),
        ])
        .add_attributes(vec![
            ("action", "unbond"),
            ("recipient", &sender),
            ("unbonded_amount", &amount.to_string()),
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
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        anchor_beth_token: deps
            .api
            .addr_humanize(&config.anchor_beth_token)?
            .to_string(),
        wormhole_beth_token: deps
            .api
            .addr_humanize(&config.wormhole_beth_token)?
            .to_string(),
    })
}
