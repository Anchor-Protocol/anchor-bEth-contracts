use beth::reward::ExecuteMsg::{DecreaseBalance, IncreaseBalance};
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw20_base::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_base::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer,
};
use cw20_base::ContractError;

use crate::state::read_reward_contract;

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_transfer(deps, env, info, recipient.to_string(), amount)?;
    Ok(Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: sender.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: recipient.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_burn(deps, env, info, amount)?;
    Ok(Response {
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: sender.to_string(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_mint(deps, env, info, recipient.to_string(), amount)?;
    Ok(Response {
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&IncreaseBalance {
                address: recipient.to_string(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: Addr,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_send(deps, env, info, contract.to_string(), amount, msg)?;
    Ok(Response {
        messages: vec![
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: reward_contract.to_string(),
                    msg: to_binary(&DecreaseBalance {
                        address: sender.to_string(),
                        amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: reward_contract.to_string(),
                    msg: to_binary(&IncreaseBalance {
                        address: contract.to_string(),
                        amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ],
            res.messages,
        ]
        .concat(),
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_transfer_from(
        deps,
        env,
        info,
        owner.to_string(),
        recipient.to_string(),
        amount,
    )?;
    Ok(Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: owner.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: recipient.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_burn_from(deps, env, info, owner.to_string(), amount)?;
    Ok(Response {
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: owner.to_string(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        ..Response::default()
    })
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    contract: Addr,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let reward_contract = deps
        .api
        .addr_humanize(&read_reward_contract(deps.storage)?)?;

    let res: Response = cw20_send_from(
        deps,
        env,
        info,
        owner.to_string(),
        contract.to_string(),
        amount,
        msg,
    )?;
    Ok(Response {
        messages: vec![
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: reward_contract.to_string(),
                    msg: to_binary(&DecreaseBalance {
                        address: owner.to_string(),
                        amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: reward_contract.to_string(),
                    msg: to_binary(&IncreaseBalance {
                        address: contract.to_string(),
                        amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ],
            res.messages,
        ]
        .concat(),
        attributes: res.attributes,
        ..Response::default()
    })
}
