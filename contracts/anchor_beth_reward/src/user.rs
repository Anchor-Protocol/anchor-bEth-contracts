use crate::state::{
    read_config, read_contract_addr, read_holder, read_holders, read_state, store_holder,
    store_state, Config, Holder, State,
};
use beth::reward::{AccruedRewardsResponse, HolderResponse, HoldersResponse};

use cosmwasm_std::{
    log, Api, BankMsg, CanonicalAddr, Coin, Decimal, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};

use crate::math::{
    decimal_multiplication_in_256, decimal_subtraction_in_256, decimal_summation_in_256,
};
use beth::deduct_tax;
use std::str::FromStr;
use terra_cosmwasm::TerraMsgWrapper;

pub fn handle_claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: Option<HumanAddr>,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let contract_addr = env.contract.address;
    let holder_addr = env.message.sender.clone();
    let holder_addr_raw = deps.api.canonical_address(&holder_addr)?;
    let recipient = match recipient {
        Some(value) => value,
        None => env.message.sender,
    };

    let mut holder: Holder = read_holder(&deps.storage, &holder_addr_raw)?;
    let mut state: State = read_state(&deps.storage)?;
    let config: Config = read_config(&deps.storage)?;

    // Load the reward contract balance
    let reward_balance = deps
        .querier
        .query_balance(contract_addr.clone(), config.reward_denom.as_str())
        .unwrap();

    // Update state's global index before calculating user rewards
    update_global_index(&mut state, reward_balance.amount)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);
    let decimals = get_decimals(all_reward_with_decimals).unwrap();

    let rewards = all_reward_with_decimals * Uint128(1);

    if rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet"));
    }

    let new_balance = (state.prev_reward_balance - rewards)?;
    state.prev_reward_balance = new_balance;
    store_state(&mut deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    store_holder(&mut deps.storage, &holder_addr_raw, &holder)?;

    Ok(HandleResponse {
        messages: vec![BankMsg::Send {
            from_address: contract_addr,
            to_address: recipient,
            amount: vec![deduct_tax(
                &deps,
                Coin {
                    denom: config.reward_denom,
                    amount: rewards,
                },
            )?],
        }
        .into()],
        log: vec![
            log("action", "claim_reward"),
            log("holder_address", holder_addr),
            log("rewards", rewards),
        ],
        data: None,
    })
}

pub fn handle_increase_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let config = read_config(&deps.storage)?;
    let token_address = assert_token_contract(config.token_contract)?;
    let address_raw = deps.api.canonical_address(&address)?;
    let sender = deps.api.canonical_address(&env.message.sender)?;

    // Check sender is token contract
    if sender != token_address {
        return Err(StdError::unauthorized());
    }

    let mut state: State = read_state(&deps.storage)?;
    let mut holder: Holder = read_holder(&deps.storage, &address_raw)?;

    // Load the reward contract balance
    let reward_balance = deps
        .querier
        .query_balance(env.contract.address, config.reward_denom.as_str())
        .unwrap();

    // Update state's global index
    update_global_index(&mut state, reward_balance.amount)?;

    // Get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance += amount;
    state.total_balance += amount;

    store_holder(&mut deps.storage, &address_raw, &holder)?;
    store_state(&mut deps.storage, &state)?;
    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "increase_balance"),
            log("holder_address", address),
            log("amount", amount),
        ],
        data: None,
    };

    Ok(res)
}

pub fn handle_decrease_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let config = read_config(&deps.storage)?;
    let token_address = assert_token_contract(config.token_contract)?;
    let address_raw = deps.api.canonical_address(&address)?;
    let sender = deps.api.canonical_address(&env.message.sender)?;

    // Check sender is token contract
    if sender != token_address {
        return Err(StdError::unauthorized());
    }

    let mut state: State = read_state(&deps.storage)?;
    let mut holder: Holder = read_holder(&deps.storage, &address_raw)?;
    if holder.balance < amount {
        return Err(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        )));
    }

    // Load the reward contract balance
    let reward_balance = deps
        .querier
        .query_balance(env.contract.address, config.reward_denom.as_str())
        .unwrap();

    // Update state's global index
    update_global_index(&mut state, reward_balance.amount)?;

    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance = (holder.balance - amount).unwrap();
    state.total_balance = (state.total_balance - amount).unwrap();

    store_holder(&mut deps.storage, &address_raw, &holder)?;
    store_state(&mut deps.storage, &state)?;
    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "decrease_balance"),
            log("holder_address", address),
            log("amount", amount),
        ],
        data: None,
    };

    Ok(res)
}

/// Increase global_index according to claimed rewards amount
fn update_global_index(state: &mut State, reward_balance: Uint128) -> StdResult<()> {
    // Zero staking balance check
    if state.total_balance.is_zero() {
        // nothing balance, skip update
        return Ok(());
    }

    // No change check
    if state.prev_reward_balance == reward_balance {
        // balance didnt change, skip update
        return Ok(());
    }

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = (reward_balance - state.prev_reward_balance)?;

    // update state
    state.prev_reward_balance = reward_balance;
    // global_index += claimed_rewards / total_balance;
    state.global_index = decimal_summation_in_256(
        state.global_index,
        Decimal::from_ratio(claimed_rewards, state.total_balance),
    );

    Ok(())
}

pub fn query_accrued_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<AccruedRewardsResponse> {
    let mut state = read_state(&deps.storage)?;
    let config = read_config(&deps.storage)?;

    let contract_addr_raw = read_contract_addr(&deps.storage)?;
    let contract_addr = deps.api.human_address(&contract_addr_raw)?;

    // Load the reward contract balance
    let reward_balance = deps
        .querier
        .query_balance(contract_addr, config.reward_denom.as_str())
        .unwrap();

    // Update state's global index
    update_global_index(&mut state, reward_balance.amount)?;

    let holder: Holder = read_holder(&deps.storage, &deps.api.canonical_address(&address)?)?;
    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;
    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);

    let rewards = all_reward_with_decimals * Uint128(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<HolderResponse> {
    let holder: Holder = read_holder(&deps.storage, &deps.api.canonical_address(&address)?)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.canonical_address(&start_after)?)
    } else {
        None
    };

    let holders: Vec<HolderResponse> = read_holders(&deps, start_after, limit)?;

    Ok(HoldersResponse { holders })
}

// calculate the reward based on the sender's index and the global index.
fn calculate_decimal_rewards(
    global_index: Decimal,
    user_index: Decimal,
    user_balance: Uint128,
) -> StdResult<Decimal> {
    let decimal_balance = Decimal::from_ratio(user_balance, Uint128(1));
    Ok(decimal_multiplication_in_256(
        decimal_subtraction_in_256(global_index, user_index),
        decimal_balance,
    ))
}

// calculate the reward with decimal
fn get_decimals(value: Decimal) -> StdResult<Decimal> {
    let stringed: &str = &*value.to_string();
    let parts: &[&str] = &*stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal::zero()),
        2 => {
            let decimals = Decimal::from_str(&*("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(StdError::generic_err("Unexpected number of dots")),
    }
}

fn assert_token_contract(token_contract: Option<CanonicalAddr>) -> StdResult<CanonicalAddr> {
    match token_contract {
        Some(v) => Ok(v),
        None => Err(StdError::generic_err("Token contract has not been set")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn proper_calculate_rewards() {
        let global_index = Decimal::from_ratio(Uint128(9), Uint128(100));
        let user_index = Decimal::zero();
        let user_balance = Uint128(1000);
        let reward = calculate_decimal_rewards(global_index, user_index, user_balance).unwrap();
        assert_eq!(reward.to_string(), "90");
    }

    #[test]
    pub fn proper_get_decimals() {
        let global_index = Decimal::from_ratio(Uint128(9999999), Uint128(100000000));
        let user_index = Decimal::zero();
        let user_balance = Uint128(10);
        let reward = get_decimals(
            calculate_decimal_rewards(global_index, user_index, user_balance).unwrap(),
        )
        .unwrap();
        assert_eq!(reward.to_string(), "0.9999999");
    }
}
