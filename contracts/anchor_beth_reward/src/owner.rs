use crate::state::{read_config, store_config};

use cosmwasm_std::{
    log, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
};
use terra_cosmwasm::TerraMsgWrapper;

pub fn handle_post_initialize<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token_contract: HumanAddr,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let mut config = read_config(&deps.storage)?;
    let owner_addr = deps.api.human_address(&config.owner)?;

    if env.message.sender != owner_addr {
        return Err(StdError::unauthorized());
    }

    config.token_contract = Some(deps.api.canonical_address(&&token_contract)?);

    store_config(&mut deps.storage, &config)?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![log("action", "post_initialize")],
        data: None,
    };

    Ok(res)
}

pub fn handle_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let mut config = read_config(&deps.storage)?;
    let owner_addr = deps.api.human_address(&config.owner)?;

    if env.message.sender != owner_addr {
        return Err(StdError::unauthorized());
    }

    config.owner = deps.api.canonical_address(&owner)?;

    store_config(&mut deps.storage, &config)?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![log("action", "update_config")],
        data: None,
    };

    Ok(res)
}
