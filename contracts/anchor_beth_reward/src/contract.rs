use crate::owner::{handle_post_initialize, handle_update_config};
use crate::state::{
    read_config, read_state, store_config, store_contract_addr, store_state, Config, State,
};
use crate::user::{
    handle_claim_rewards, handle_decrease_balance, handle_increase_balance, query_accrued_rewards,
    query_holder, query_holders,
};
use beth::reward::{ConfigResponse, HandleMsg, InitMsg, QueryMsg, StateResponse};
use cosmwasm_std::{
    to_binary, Api, Binary, Decimal, Env, Extern, HandleResponse, InitResponse, Querier, StdResult,
    Storage, Uint128,
};

use terra_cosmwasm::TerraMsgWrapper;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let conf = Config {
        owner: deps.api.canonical_address(&msg.owner)?,
        reward_denom: msg.reward_denom,
        token_contract: None,
    };

    store_config(&mut deps.storage, &conf)?;
    store_state(
        &mut deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    // keep contract address in state to be able to use it in queries
    store_contract_addr(
        &mut deps.storage,
        &deps.api.canonical_address(&env.contract.address)?,
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    match msg {
        HandleMsg::ClaimRewards { recipient } => handle_claim_rewards(deps, env, recipient),
        HandleMsg::PostInitialize { token_contract } => {
            handle_post_initialize(deps, env, token_contract)
        }
        HandleMsg::UpdateConfig { owner } => handle_update_config(deps, env, owner),
        HandleMsg::IncreaseBalance { address, amount } => {
            handle_increase_balance(deps, env, address, amount)
        }
        HandleMsg::DecreaseBalance { address, amount } => {
            handle_decrease_balance(deps, env, address, amount)
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(&deps)?),
        QueryMsg::State {} => to_binary(&query_state(&deps)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(&deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(&deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(&deps, start_after, limit)?)
        }
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let config: Config = read_config(&deps.storage)?;
    let mut res = ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        reward_denom: config.reward_denom,
        token_contract: None,
    };

    if let Some(token_contract) = config.token_contract {
        res.token_contract = Some(deps.api.human_address(&&token_contract)?);
    }

    Ok(res)
}

fn query_state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<StateResponse> {
    let state: State = read_state(&deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}
