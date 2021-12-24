use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Attribute, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg,
};

use crate::contract::{execute, instantiate, query};
use crate::testing::mock_querier::mock_dependencies;
use beth::converter::Cw20HookMsg::{ConvertAnchorToWormhole, ConvertWormholeToAnchor};
use beth::converter::ExecuteMsg::{Receive, RegisterTokens};
use beth::converter::{ConfigResponse, InstantiateMsg, QueryMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

const MOCK_OWNER_ADDR: &str = "owner0000";
const MOCK_ANCHOR_TOKEN_CONTRACT_ADDR: &str = "beth_token0000";
const MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR: &str = "wormhole_token0000";

fn default_init() -> InstantiateMsg {
    InstantiateMsg {
        owner: MOCK_OWNER_ADDR.to_string(),
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
            anchor_token_address: None,
            wormhole_token_address: None,
        }
    );
}

#[test]
fn proper_convert_to_anchor() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_config = RegisterTokens {
        anchor_token_address: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_token_address: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
    };

    // set anchor and wormhole decimals
    deps.querier.set_decimals(6, 8);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_OWNER_ADDR, &[]),
        update_config,
    )
    .unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "register_token_contracts")
    );

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(100000000),
        msg: to_binary(&ConvertWormholeToAnchor {}).unwrap(),
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
            contract_addr: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                // 100000000 / 10^2 = 1000000
                amount: Uint128::new(1000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );

    //cannot convert less than 100 micro wormhole
    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(1),
        msg: to_binary(&ConvertWormholeToAnchor {}).unwrap(),
    });

    // unauthorized request
    let invalid_info = mock_info("invalid", &[]);
    let error_res =
        execute(deps.as_mut(), mock_env(), invalid_info, receive_msg.clone()).unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    // successful request
    let wormhole_info = mock_info(MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), wormhole_info, receive_msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("cannot convert; conversion is only possible for amounts greater than 100 wormhole token")
    );
}

#[test]
fn proper_convert_to_wormhole() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_config = RegisterTokens {
        anchor_token_address: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_token_address: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
    };

    // set anchor and wormhole decimals
    deps.querier.set_decimals(6, 8);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_OWNER_ADDR, &[]),
        update_config,
    )
    .unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "register_token_contracts")
    );

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(100000000),
        msg: to_binary(&ConvertAnchorToWormhole {}).unwrap(),
    });

    // unauthorized request
    let invalid_info = mock_info("invalid", &[]);
    let error_res =
        execute(deps.as_mut(), mock_env(), invalid_info, receive_msg.clone()).unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    // successful
    let beth_info = mock_info(MOCK_ANCHOR_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), beth_info, receive_msg).unwrap();
    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                // 100000000 * 10^2 = 10000000000
                amount: Uint128::new(10000000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );
    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(100000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn proper_convert_to_anchor_with_more_decimals() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_config = RegisterTokens {
        anchor_token_address: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_token_address: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
    };

    // set anchor and wormhole decimals
    deps.querier.set_decimals(10, 8);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_OWNER_ADDR, &[]),
        update_config,
    )
    .unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "register_token_contracts")
    );

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(100000000),
        msg: to_binary(&ConvertWormholeToAnchor {}).unwrap(),
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
            contract_addr: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: sender.to_string(),
                //anchor decimals is bigger than wormhole then we should multiply with 10^2
                // 100000000 * 10^2 = 10000000000
                amount: Uint128::new(10000000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn proper_convert_to_wormhole_with_less_decimals() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let sender = "addr0000";
    let info = mock_info(sender, &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_config = RegisterTokens {
        anchor_token_address: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_token_address: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
    };

    // set anchor and wormhole decimals
    deps.querier.set_decimals(10, 8);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_OWNER_ADDR, &[]),
        update_config,
    )
    .unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "register_token_contracts")
    );

    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(100000000),
        msg: to_binary(&ConvertAnchorToWormhole {}).unwrap(),
    });

    // unauthorized request
    let invalid_info = mock_info("invalid", &[]);
    let error_res =
        execute(deps.as_mut(), mock_env(), invalid_info, receive_msg.clone()).unwrap_err();
    assert_eq!(error_res, StdError::generic_err("unauthorized"));

    // successful
    let beth_info = mock_info(MOCK_ANCHOR_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), beth_info, receive_msg).unwrap();
    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                //anchor decimals is bigger than wormhole then we should divide with 10^2
                // 100000000 * 10^2 = 1000000
                amount: Uint128::new(1000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );
    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(100000000)
            })
            .unwrap(),
            funds: vec![]
        }))
    );

    //cannot convert less than 100 micro wormhole
    let receive_msg = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(1),
        msg: to_binary(&ConvertAnchorToWormhole {}).unwrap(),
    });

    // successful request
    let wormhole_info = mock_info(MOCK_ANCHOR_TOKEN_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), wormhole_info, receive_msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err(
            "cannot convert; conversion is only possible for amounts greater than 100 anchor token"
        )
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

    let update_config = RegisterTokens {
        anchor_token_address: MOCK_ANCHOR_TOKEN_CONTRACT_ADDR.to_string(),
        wormhole_token_address: MOCK_WORMHOLE_TOKEN_CONTRACT_ADDR.to_string(),
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
    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "register_token_contracts")
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            owner: MOCK_OWNER_ADDR.to_string(),
            anchor_token_address: Some("beth_token0000".to_string()),
            wormhole_token_address: Some("wormhole_token0000".to_string()),
        }
    );
}
