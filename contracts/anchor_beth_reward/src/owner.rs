use crate::state::{read_config, store_config};

use cosmwasm_std::{attr, Addr, DepsMut, MessageInfo, Response, StdError, StdResult};
use terra_cosmwasm::TerraMsgWrapper;

pub fn execute_post_initialize(
    deps: DepsMut,
    info: MessageInfo,
    token_contract: Addr,
) -> StdResult<Response<TerraMsgWrapper>> {
    let mut config = read_config(deps.storage)?;
    let owner_addr = deps.api.addr_humanize(&config.owner)?;

    if info.sender != owner_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    config.token_contract = Some(deps.api.addr_canonicalize(token_contract.as_str())?);

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "post_initialize")]))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Addr,
) -> StdResult<Response<TerraMsgWrapper>> {
    let mut config = read_config(deps.storage)?;
    let owner_addr = deps.api.addr_humanize(&config.owner)?;

    if info.sender != owner_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    config.owner = deps.api.addr_canonicalize(owner.as_str())?;

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}
