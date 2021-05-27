use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, BankMsg, Coin, CosmosMsg, Decimal, HumanAddr, StdError, Uint128};

use crate::contract::{handle, init, query};
use crate::math::{decimal_multiplication_in_256, decimal_subtraction_in_256};
use crate::testing::mock_querier::mock_dependencies;
use beth::reward::{
    ConfigResponse, HandleMsg, HolderResponse, HoldersResponse, InitMsg, QueryMsg, StateResponse,
};
use std::str::FromStr;

const DEFAULT_REWARD_DENOM: &str = "uusd";
const MOCK_OWNER_ADDR: &str = "owner0000";
const MOCK_TOKEN_CONTRACT_ADDR: &str = "token0000";

fn default_init() -> InitMsg {
    InitMsg {
        owner: HumanAddr::from(MOCK_OWNER_ADDR),
        reward_denom: DEFAULT_REWARD_DENOM.to_string(),
    }
}

#[test]
fn proper_init() {
    let mut deps = mock_dependencies(20, &[]);
    let init_msg = default_init();

    let env = mock_env("addr0000", &[]);

    let res = init(&mut deps, env, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(&deps, QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            owner: HumanAddr::from(MOCK_OWNER_ADDR),
            token_contract: Some(HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR)),
            reward_denom: DEFAULT_REWARD_DENOM.to_string(),
        }
    );

    let res = query(&deps, QueryMsg::State {}).unwrap();
    let state_response: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state_response,
        StateResponse {
            global_index: Decimal::zero(),
            total_balance: Uint128(0u128),
            prev_reward_balance: Uint128::zero()
        }
    );
}

#[test]
fn increase_balance() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(100u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg.clone());
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(100u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(200u128),
            index: Decimal::one(),
            pending_rewards: Decimal::from_str("100").unwrap(),
        }
    );
}

#[test]
fn increase_balance_with_decimals() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(100000u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(11u128),
    };

    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg.clone());
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(11u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(10u128),
    };
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    let index = decimal_multiplication_in_256(
        Decimal::from_ratio(Uint128(100000), Uint128(11)),
        Decimal::one(),
    );
    let user_pend_reward = decimal_multiplication_in_256(
        Decimal::from_str("11").unwrap(),
        decimal_subtraction_in_256(holder_response.index, Decimal::zero()),
    );
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(21u128),
            index,
            pending_rewards: user_pend_reward,
        }
    );
}

#[test]
fn decrease_balance() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(100u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::DecreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    // Failed unautorized
    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg.clone());
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    };

    // Failed underflow
    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Decrease amount cannot exceed user balance: 0")
        }
        _ => panic!("DO NOT ENTER HERE"),
    };

    // Increase balance first
    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env, msg).unwrap();

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = HandleMsg::DecreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::zero(),
            index: Decimal::one(),
            pending_rewards: Decimal::from_str("100").unwrap(),
        }
    );
}

#[test]
fn claim_rewards() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(100u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(100u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let msg = HandleMsg::ClaimRewards { recipient: None };
    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("addr0000"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99u128), // 1% tax
            },]
        })]
    );

    let msg = HandleMsg::ClaimRewards {
        recipient: Some(HumanAddr::from("addr0001")),
    };
    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("addr0001"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99u128), // 1% tax
            },]
        })]
    );
}

#[test]
fn claim_rewards_with_decimals() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(99999u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(11u128),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(11u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let msg = HandleMsg::ClaimRewards { recipient: None };
    let env = mock_env("addr0000", &[]);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("addr0000"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99007u128), // 1% tax
            },]
        })]
    );

    let res = query(
        &deps,
        QueryMsg::Holder {
            address: HumanAddr::from("addr0000"),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    let index = decimal_multiplication_in_256(
        Decimal::from_ratio(Uint128(99999), Uint128(11)),
        Decimal::one(),
    );
    assert_eq!(
        holder_response,
        HolderResponse {
            address: HumanAddr::from("addr0000"),
            balance: Uint128::from(11u128),
            index,
            pending_rewards: Decimal::from_str("0.999999999999999991").unwrap(),
        }
    );

    let res = query(&deps, QueryMsg::State {}).unwrap();
    let state_response: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state_response,
        StateResponse {
            global_index: index,
            total_balance: Uint128(11u128),
            prev_reward_balance: Uint128(1)
        }
    );
}

#[test]
fn query_holders() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(100u128),
        }],
    );

    let init_msg = default_init();
    let env = mock_env("addr0000", &[]);
    init(&mut deps, env, init_msg).unwrap();

    let msg = HandleMsg::PostInitialize {
        token_contract: HumanAddr::from(MOCK_TOKEN_CONTRACT_ADDR),
    };
    let env = mock_env(MOCK_OWNER_ADDR, &[]);
    let _res = handle(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    let env = mock_env(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    handle(&mut deps, env.clone(), msg).unwrap();

    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0001"),
        amount: Uint128::from(200u128),
    };

    handle(&mut deps, env.clone(), msg).unwrap();
    let msg = HandleMsg::IncreaseBalance {
        address: HumanAddr::from("addr0002"),
        amount: Uint128::from(300u128),
    };

    handle(&mut deps, env, msg).unwrap();

    let res = query(
        &deps,
        QueryMsg::Holders {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let holders_response: HoldersResponse = from_binary(&res).unwrap();
    assert_eq!(
        holders_response,
        HoldersResponse {
            holders: vec![
                HolderResponse {
                    address: HumanAddr::from("addr0000"),
                    balance: Uint128::from(100u128),
                    index: Decimal::zero(), // first one skips update
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: HumanAddr::from("addr0001"),
                    balance: Uint128::from(200u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: HumanAddr::from("addr0002"),
                    balance: Uint128::from(300u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                }
            ],
        }
    );

    // Set limit
    let res = query(
        &deps,
        QueryMsg::Holders {
            start_after: None,
            limit: Some(1),
        },
    )
    .unwrap();
    let holders_response: HoldersResponse = from_binary(&res).unwrap();
    assert_eq!(
        holders_response,
        HoldersResponse {
            holders: vec![HolderResponse {
                address: HumanAddr::from("addr0000"),
                balance: Uint128::from(100u128),
                index: Decimal::zero(),
                pending_rewards: Decimal::zero(),
            }],
        }
    );

    // Set start_after
    let res = query(
        &deps,
        QueryMsg::Holders {
            start_after: Some(HumanAddr::from("addr0000")),
            limit: None,
        },
    )
    .unwrap();
    let holders_response: HoldersResponse = from_binary(&res).unwrap();
    assert_eq!(
        holders_response,
        HoldersResponse {
            holders: vec![
                HolderResponse {
                    address: HumanAddr::from("addr0001"),
                    balance: Uint128::from(200u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: HumanAddr::from("addr0002"),
                    balance: Uint128::from(300u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                }
            ],
        }
    );

    // Set start_after and limit
    let res = query(
        &deps,
        QueryMsg::Holders {
            start_after: Some(HumanAddr::from("addr0000")),
            limit: Some(1),
        },
    )
    .unwrap();
    let holders_response: HoldersResponse = from_binary(&res).unwrap();
    assert_eq!(
        holders_response,
        HoldersResponse {
            holders: vec![HolderResponse {
                address: HumanAddr::from("addr0001"),
                balance: Uint128::from(200u128),
                index: Decimal::one(),
                pending_rewards: Decimal::zero(),
            }],
        }
    );
}
