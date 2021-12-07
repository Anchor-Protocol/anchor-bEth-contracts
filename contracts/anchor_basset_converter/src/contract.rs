#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{
    read_config, store_asset, store_asset_anchor, store_asset_wormhole, store_config, Config,
    ANCHOR_WHITELISTED_ASSETS, WHITELISTED_ASSETS, WORMHOLE_WHITELISTED_ASSETS,
};

use beth::converter::{
    Asset, ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    WhitelistedAssetResponse, WhitelistedAssetsResponse,
};
use cosmwasm_std::{
    from_binary, to_binary, to_vec, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_storage_plus::Bound;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
    };

    store_config(deps.storage).save(&conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::WhitelisteAsset { asset } => execute_whitelist_asset(deps, info, asset),
    }
}

/// CW20 token receive handler.
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::ConvertWormholeToAnchor {}) => {
            // only wormhole token contract can execute this message
            let asset_raw = deps.api.addr_canonicalize(contract_addr.as_str()).unwrap();
            let asset = WORMHOLE_WHITELISTED_ASSETS.load(deps.storage, asset_raw.as_slice());
            match asset {
                Ok(address) => execute_convert_to_anchor(cw20_msg.amount, cw20_msg.sender, address),
                Err(_) => Err(StdError::generic_err("Asset is not register")),
            }
        }
        Ok(Cw20HookMsg::ConvertAnchorToWormhole {}) => {
            // only anchor token contract can execute this message
            let asset_raw = deps.api.addr_canonicalize(contract_addr.as_str()).unwrap();
            let asset = ANCHOR_WHITELISTED_ASSETS.load(deps.storage, asset_raw.as_slice());

            match asset {
                Ok(address) => execute_convert_to_wormhole(
                    cw20_msg.amount,
                    cw20_msg.sender,
                    contract_addr.to_string(),
                    address,
                ),
                Err(_) => Err(StdError::generic_err("Asset is not register")),
            }
        }
        Err(err) => Err(err),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> StdResult<Response> {
    let api = deps.api;

    let mut config = read_config(deps.storage)?;
    if config.owner != api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        config.owner = api.addr_canonicalize(&owner)?;
    }

    store_config(deps.storage).save(&config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn execute_whitelist_asset(
    deps: DepsMut,
    info: MessageInfo,
    asset: Asset,
) -> StdResult<Response> {
    let api = deps.api;

    let config = read_config(deps.storage)?;
    if config.owner != api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // store the token pair with the key being wormhole token address
    store_asset_wormhole(deps.storage, deps.api, &asset)?;

    // store the token pair with the key being anchor token address
    store_asset_anchor(deps.storage, deps.api, &asset)?;

    // store for query
    store_asset(deps.storage, &asset)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "register_asset"),
        (
            "wormhole_token_address",
            asset.wormhole_token_address.as_str(),
        ),
        ("anchor_token_address", asset.anchor_token_address.as_str()),
    ]))
}

pub(crate) fn execute_convert_to_anchor(
    amount: Uint128,
    sender: String,
    anchor_token_address: String,
) -> StdResult<Response> {
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: anchor_token_address,
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

pub(crate) fn execute_convert_to_wormhole(
    amount: Uint128,
    sender: String,
    anchor_token_address: String,
    wormhole_token_address: String,
) -> StdResult<Response> {
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: anchor_token_address,
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: wormhole_token_address,
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: sender.clone(),
                    amount,
                })?,
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
        QueryMsg::WhitelistedAsset { asset_name } => {
            to_binary(&query_whitelisted_asset(deps, asset_name)?)
        }
        QueryMsg::WhitelistedAssets { start_after, limit } => {
            to_binary(&query_whitelisted_assets(deps, start_after, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    })
}

fn query_whitelisted_asset(deps: Deps, asset_name: String) -> StdResult<WhitelistedAssetResponse> {
    let asset = WHITELISTED_ASSETS.load(deps.storage, to_vec(&asset_name)?.as_slice())?;
    Ok(WhitelistedAssetResponse {
        asset: Asset {
            asset_name,
            wormhole_token_address: asset.wormhole_token_address,
            anchor_token_address: asset.anchor_token_address,
        },
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_whitelisted_assets(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<WhitelistedAssetsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after)?.map(Bound::exclusive);

    let assets: StdResult<Vec<Asset>> = WHITELISTED_ASSETS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, asset) = item?;
            Ok(Asset {
                asset_name: asset.asset_name,
                wormhole_token_address: asset.wormhole_token_address,
                anchor_token_address: asset.anchor_token_address,
            })
        })
        .collect();
    Ok(WhitelistedAssetsResponse { assets: assets? })
}

pub fn calc_range_start(start_after: Option<String>) -> StdResult<Option<Vec<u8>>> {
    match start_after {
        Some(_) => {
            let mut v: Vec<u8> = to_vec(&start_after)?;
            v.push(0);
            Ok(Some(v))
        }
        None => Ok(None),
    }
}
