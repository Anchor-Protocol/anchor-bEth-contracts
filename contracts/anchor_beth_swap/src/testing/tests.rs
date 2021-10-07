use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Attribute, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg,
};

use crate::contract::{execute, instantiate, query};
use beth::swap::Cw20HookMsg::{Bond, Unbond};
use beth::swap::ExecuteMsg::{Receive, UpdateConfig};
use beth::swap::{ConfigResponse, InstantiateMsg, QueryMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

const MOCK_OWNER_ADDR: &str = "owner0000";
const MOCK_BETH_TOKEN_CONTRACT_ADDR: &str = "beth_token0000";
const MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR: &str = "wormhole_token0000";
const DEFAULT_BOND: Uint128 = Uint128::new(10000);

fn default_init() -> InstantiateMsg {
    InstantiateMsg {
        owner: MOCK_OWNER_ADDR.to_string(),
        anchor_beth_token: MOCK_BETH_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_beth_token: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
    }
}

#[test]
fn proper_init() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            owner: MOCK_OWNER_ADDR.to_string(),
            anchor_beth_token: MOCK_BETH_TOKEN_CONTRACT_ADDR.to_string(),
            wormhole_beth_token: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
        }
    );
}

#[test]
fn proper_bond() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: DEFAULT_BOND,
        msg: to_binary(&Bond {}).unwrap(),
    });

    // unauthorized request
    let invalid_info = mock_info("invalid", &[]);
    let error_res =
        execute(deps.as_mut(), mock_env(), invalid_info, receive_msg.clone()).unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    // successful request
    let wormhole_info = mock_info(MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), wormhole_info, receive_msg).unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_BETH_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                amount: DEFAULT_BOND
            })
            .unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn proper_unbond() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: DEFAULT_BOND,
        msg: to_binary(&Unbond {}).unwrap(),
    });

    // unauthorized request
    let invalid_info = mock_info("invalid", &[]);
    let error_res =
        execute(deps.as_mut(), mock_env(), invalid_info, receive_msg.clone()).unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    // successful
    let beth_info = mock_info(MOCK_BETH_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), beth_info, receive_msg).unwrap();
    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount: DEFAULT_BOND
            })
            .unwrap(),
            funds: vec![]
        }))
    );
    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_BETH_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: DEFAULT_BOND
            })
            .unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn proper_update_config() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let invalid_info = mock_info("invalid", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_config = UpdateConfig {
        owner: Some("new_owner".to_string()),
        anchor_beth_token: Some("new_beth_token".to_string()),
        wormhole_beth_token: Some("new_wormhole_token".to_string()),
    };

    // unauthorized request
    let error_res = execute(
        deps.as_mut(),
        mock_env(),
        invalid_info,
        update_config.clone(),
    )
    .unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    //successful one
    let res = execute(deps.as_mut(), mock_env(), info, update_config).unwrap();
    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            owner: "new_owner".to_string(),
            anchor_beth_token: "new_beth_token".to_string(),
            wormhole_beth_token: "new_wormhole_token".to_string(),
        }
    );
}
