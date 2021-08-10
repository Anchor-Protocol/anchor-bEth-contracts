use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coins, to_binary, Api, CosmosMsg, DepsMut, OwnedDeps, Querier, Storage, SubMsg, Uint128,
    WasmMsg,
};

use beth::reward::ExecuteMsg::{DecreaseBalance, IncreaseBalance};
use cw20::{Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};
use cw20_legacy::contract::{query_minter, query_token_info};
use cw20_legacy::msg::ExecuteMsg;

use crate::contract::{execute, instantiate};
use crate::msg::TokenInstantiateMsg;
use crate::state::read_reward_contract;

use std::borrow::BorrowMut;

const MOCK_REWARD_CONTRACT_ADDR: &str = "bethreward0000";
const MOCK_MINTER_ADDR: &str = "minter0000";

// this will set up the init for other tests
fn do_init_with_minter<S: Storage, A: Api, Q: Querier>(
    deps: &mut OwnedDeps<S, A, Q>,
    minter: String,
    cap: Option<Uint128>,
) -> TokenInfoResponse {
    _do_init(deps, Some(MinterResponse { minter, cap }))
}

// this will set up the init for other tests
fn _do_init<S: Storage, A: Api, Q: Querier>(
    deps: &mut OwnedDeps<S, A, Q>,
    mint: Option<MinterResponse>,
) -> TokenInfoResponse {
    let reward_contract = MOCK_REWARD_CONTRACT_ADDR.to_string();
    let init_msg = TokenInstantiateMsg {
        name: "bluna".to_string(),
        symbol: "BLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: mint.clone(),
        reward_contract,
    };

    let info = mock_info(MOCK_REWARD_CONTRACT_ADDR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "bluna".to_string(),
            symbol: "BLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );
    assert_eq!(query_minter(deps.as_ref()).unwrap(), mint);
    meta
}

pub fn do_mint(deps: DepsMut, addr: String, amount: Uint128) {
    let msg = ExecuteMsg::Mint {
        recipient: addr,
        amount,
    };
    let info = mock_info(MOCK_MINTER_ADDR, &[]);
    let res = execute(deps, mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let reward_contract = MOCK_REWARD_CONTRACT_ADDR.to_string();
    let reward_contract_raw = deps.api.addr_canonicalize(&reward_contract).unwrap();

    let init_msg = TokenInstantiateMsg {
        name: "bluna".to_string(),
        symbol: "BLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: None,
        reward_contract: reward_contract.clone(),
    };
    let info = mock_info(&reward_contract, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "bluna".to_string(),
            symbol: "BLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );

    assert_eq!(
        read_reward_contract(&deps.storage).unwrap(),
        reward_contract_raw
    );
}

#[test]
fn transfer() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: addr1,
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: addr2,
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
}

#[test]
fn transfer_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let addr3 = "addr0003".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr3.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info(&addr3, &[]);
    let msg = ExecuteMsg::TransferFrom {
        owner: addr1.clone(),
        recipient: addr2.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: addr1,
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: addr2,
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
}

#[test]
fn mint() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = "addr0000".to_string();

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);

    let info = mock_info(MOCK_MINTER_ADDR, &[]);
    let msg = ExecuteMsg::Mint {
        recipient: addr.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
            msg: to_binary(&IncreaseBalance {
                address: addr,
                amount: Uint128::new(1u128),
            })
            .unwrap(),
            funds: vec![],
        })),]
    );
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = "addr0000".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr.clone(), amount1);

    let info = mock_info(&addr, &[]);
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: addr,
                amount: Uint128::new(1u128),
            })
            .unwrap(),
            funds: vec![],
        })),]
    );
}

#[test]
fn burn_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = "addr0000".to_string();
    let addr1 = "addr0001".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr.clone(), amount1);

    let info = mock_info(&addr, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr1.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::BurnFrom {
        owner: addr.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: addr,
                amount: Uint128::new(1u128),
            })
            .unwrap(),
            funds: vec![],
        })),]
    );
}

#[test]
fn send() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = "addr0001".to_string();
    let dummny_contract_addr = "dummy".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::Send {
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0..2].to_vec(),
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: addr1.clone(),
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: dummny_contract_addr.clone(),
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
    assert_eq!(
        res.messages[2],
        SubMsg::new(
            Cw20ReceiveMsg {
                sender: addr1,
                amount: Uint128::new(1),
                msg: to_binary(&dummy_msg).unwrap(),
            }
            .into_cosmos_msg(dummny_contract_addr)
            .unwrap()
        )
    );
}

#[test]
fn send_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let dummny_contract_addr = "dummy".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), MOCK_MINTER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr2.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(&addr2, &[]);
    let msg = ExecuteMsg::SendFrom {
        owner: addr1.clone(),
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0..2].to_vec(),
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: addr1,
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARD_CONTRACT_ADDR.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: dummny_contract_addr.clone(),
                    amount: Uint128::new(1u128),
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );

    assert_eq!(
        res.messages[2],
        SubMsg::new(
            Cw20ReceiveMsg {
                sender: addr2,
                amount: Uint128::new(1),
                msg: to_binary(&dummy_msg).unwrap(),
            }
            .into_cosmos_msg(dummny_contract_addr)
            .unwrap()
        )
    );
}
