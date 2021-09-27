#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::owner::{execute_post_initialize, execute_update_config};
use crate::state::{
    read_config, read_state, store_config, store_contract_addr, store_state, Config, State,
};
use crate::user::{
    execute_claim_rewards, execute_decrease_balance, execute_increase_balance,
    query_accrued_rewards, query_holder, query_holders,
};
use beth::reward::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};

use terra_cosmwasm::TerraMsgWrapper;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        reward_denom: msg.reward_denom,
        token_contract: None,
    };

    store_config(deps.storage, &conf)?;
    store_state(
        deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    // keep contract address in state to be able to use it in queries
    store_contract_addr(
        deps.storage,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<TerraMsgWrapper>> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => {
            let api = deps.api;
            execute_claim_rewards(deps, env, info, optional_addr_validate(api, recipient)?)
        }
        ExecuteMsg::PostInitialize { token_contract } => {
            let token_addr = deps.api.addr_validate(&token_contract)?;
            execute_post_initialize(deps, info, token_addr)
        }
        ExecuteMsg::UpdateConfig { owner } => {
            let owner_addr = deps.api.addr_validate(&owner)?;
            execute_update_config(deps, info, owner_addr)
        }
        ExecuteMsg::IncreaseBalance { address, amount } => {
            let addr = deps.api.addr_validate(&address)?;
            execute_increase_balance(deps, env, info, addr, amount)
        }
        ExecuteMsg::DecreaseBalance { address, amount } => {
            let addr = deps.api.addr_validate(&address)?;
            execute_decrease_balance(deps, env, info, addr, amount)
        }
    }
}

fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<Addr>> {
    let addr = if let Some(addr) = addr {
        Some(api.addr_validate(&addr)?)
    } else {
        None
    };

    Ok(addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::AccruedRewards { address } => {
            let addr = deps.api.addr_validate(&address)?;
            to_binary(&query_accrued_rewards(deps, addr)?)
        }
        QueryMsg::Holder { address } => {
            let addr = deps.api.addr_validate(&address)?;
            to_binary(&query_holder(deps, addr)?)
        }
        QueryMsg::Holders { start_after, limit } => {
            let api = deps.api;
            to_binary(&query_holders(
                deps,
                optional_addr_validate(api, start_after)?,
                limit,
            )?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    let mut res = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        reward_denom: config.reward_denom,
        token_contract: None,
    };

    if let Some(token_contract) = config.token_contract {
        res.token_contract = Some(deps.api.addr_humanize(&token_contract)?.to_string());
    }

    Ok(res)
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}
