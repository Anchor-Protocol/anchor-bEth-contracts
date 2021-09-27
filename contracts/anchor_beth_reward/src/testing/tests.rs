use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, BankMsg, Coin, CosmosMsg, Decimal, StdError, SubMsg, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::math::{decimal_multiplication_in_256, decimal_subtraction_in_256};
use crate::testing::mock_querier::mock_dependencies;
use beth::reward::{
    ConfigResponse, ExecuteMsg, HolderResponse, HoldersResponse, InstantiateMsg, QueryMsg,
    StateResponse,
};
use std::str::FromStr;

const DEFAULT_REWARD_DENOM: &str = "uusd";
const MOCK_OWNER_ADDR: &str = "owner0000";
const MOCK_TOKEN_CONTRACT_ADDR: &str = "token0000";

fn default_init() -> InstantiateMsg {
    InstantiateMsg {
        owner: MOCK_OWNER_ADDR.to_string(),
        reward_denom: DEFAULT_REWARD_DENOM.to_string(),
    }
}

#[test]
fn proper_init() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            owner: MOCK_OWNER_ADDR.to_string(),
            token_contract: Some(MOCK_TOKEN_CONTRACT_ADDR.to_string()),
            reward_denom: DEFAULT_REWARD_DENOM.to_string(),
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state_response: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state_response,
        StateResponse {
            global_index: Decimal::zero(),
            total_balance: Uint128::new(0u128),
            prev_reward_balance: Uint128::zero()
        }
    );
}

#[test]
fn increase_balance() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(100u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(200u128),
            index: Decimal::one(),
            pending_rewards: Decimal::from_str("100").unwrap(),
        }
    );
}

#[test]
fn increase_balance_with_decimals() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100000u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(11u128),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(11u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(10u128),
    };
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    let index = decimal_multiplication_in_256(
        Decimal::from_ratio(Uint128::new(100000), Uint128::new(11)),
        Decimal::one(),
    );
    let user_pend_reward = decimal_multiplication_in_256(
        Decimal::from_str("11").unwrap(),
        decimal_subtraction_in_256(holder_response.index, Decimal::zero()),
    );
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(21u128),
            index,
            pending_rewards: user_pend_reward,
        }
    );
}

#[test]
fn decrease_balance() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::DecreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };

    // Failed unautorized
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    };

    // Failed underflow
    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Decrease amount cannot exceed user balance: 0")
        }
        _ => panic!("DO NOT ENTER HERE"),
    };

    // Increase balance first
    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    let msg = ExecuteMsg::DecreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::zero(),
            index: Decimal::one(),
            pending_rewards: Decimal::from_str("100").unwrap(),
        }
    );
}

#[test]
fn claim_rewards() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(100u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let msg = ExecuteMsg::ClaimRewards { recipient: None };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99u128), // 1% tax
            },]
        }))]
    );

    let msg = ExecuteMsg::ClaimRewards {
        recipient: Some("addr0001".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99u128), // 1% tax
            },]
        }))]
    );
}

#[test]
fn claim_rewards_with_decimals() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(99999u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: "addr0000".to_string(),
        amount: Uint128::from(11u128),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(11u128),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }
    );

    let msg = ExecuteMsg::ClaimRewards { recipient: None };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99007u128), // 1% tax
            },]
        }))]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holder {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let holder_response: HolderResponse = from_binary(&res).unwrap();
    let index = decimal_multiplication_in_256(
        Decimal::from_ratio(Uint128::new(99999), Uint128::new(11)),
        Decimal::one(),
    );
    assert_eq!(
        holder_response,
        HolderResponse {
            address: "addr0000".to_string(),
            balance: Uint128::from(11u128),
            index,
            pending_rewards: Decimal::from_str("0.999999999999999991").unwrap(),
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state_response: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state_response,
        StateResponse {
            global_index: index,
            total_balance: Uint128::new(11u128),
            prev_reward_balance: Uint128::new(1)
        }
    );
}

#[test]
fn query_holders() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let msg = ExecuteMsg::PostInitialize {
        token_contract: MOCK_TOKEN_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: String::from("addr0000"),
        amount: Uint128::from(100u128),
    };

    let info = mock_info(MOCK_TOKEN_CONTRACT_ADDR, &[]);
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::IncreaseBalance {
        address: String::from("addr0001"),
        amount: Uint128::from(200u128),
    };

    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let msg = ExecuteMsg::IncreaseBalance {
        address: String::from("addr0002"),
        amount: Uint128::from(300u128),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
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
                    address: String::from("addr0000"),
                    balance: Uint128::from(100u128),
                    index: Decimal::zero(),
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: String::from("addr0001"),
                    balance: Uint128::from(200u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: String::from("addr0002"),
                    balance: Uint128::from(300u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                },
            ],
        }
    );

    // Set limit
    let res = query(
        deps.as_ref(),
        mock_env(),
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
                address: String::from("addr0000"),
                balance: Uint128::from(100u128),
                index: Decimal::zero(),
                pending_rewards: Decimal::zero(),
            }],
        }
    );

    // Set start_after
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holders {
            start_after: Some(String::from("addr0000")),
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
                    address: String::from("addr0001"),
                    balance: Uint128::from(200u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                },
                HolderResponse {
                    address: String::from("addr0002"),
                    balance: Uint128::from(300u128),
                    index: Decimal::one(),
                    pending_rewards: Decimal::zero(),
                }
            ],
        }
    );

    // Set start_after and limit
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Holders {
            start_after: Some(String::from("addr0000")),
            limit: Some(1),
        },
    )
    .unwrap();
    let holders_response: HoldersResponse = from_binary(&res).unwrap();
    assert_eq!(
        holders_response,
        HoldersResponse {
            holders: vec![HolderResponse {
                address: String::from("addr0001"),
                balance: Uint128::from(200u128),
                index: Decimal::one(),
                pending_rewards: Decimal::zero(),
            }],
        }
    );
}
