#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use cw20_legacy::allowances::{execute_decrease_allowance, execute_increase_allowance};
use cw20_legacy::contract::instantiate as cw20_instantiate;
use cw20_legacy::contract::query as cw20_query;
use cw20_legacy::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::handler::*;
use crate::msg::{MigrateMsg, TokenInstantiateMsg};
use crate::state::store_reward_contract;
use cw20_legacy::state::{MinterData, TOKEN_INFO};
use cw20_legacy::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: TokenInstantiateMsg,
) -> StdResult<Response> {
    let reward_raw = deps.api.addr_canonicalize(&msg.reward_contract)?;
    store_reward_contract(deps.storage, &reward_raw)?;

    cw20_instantiate(
        deps,
        env,
        info,
        InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            mint: msg.mint,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, _env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let mut token_info = TOKEN_INFO.load(deps.storage)?;

    let minter = MinterData {
        minter: deps.api.addr_canonicalize(&msg.minter)?,
        cap: None,
    };
    token_info.mint = Some(minter);
    TOKEN_INFO.save(deps.storage, &token_info)?;
    Ok(Response::default())
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Api};
    use cw20::MinterResponse;

    #[test]
    fn proper_migrate() {
        let mut deps = mock_dependencies(&[]);
        let first_minter = "first_minter";
        let new_minter = "new_minter";

        let init_msg = TokenInstantiateMsg {
            name: "bonded ETH".to_string(),
            symbol: "BETH".to_string(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: first_minter.to_string(),
                cap: None,
            }),
            reward_contract: "reward_contract".to_string(),
        };

        let info = mock_info("sender", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        //migrate
        let migrate_msg = MigrateMsg {
            minter: new_minter.to_string(),
        };
        let res = migrate(deps.as_mut(), mock_env(), migrate_msg).unwrap();
        assert_eq!(res, Response::default());

        let token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            Addr::unchecked(new_minter),
            deps.api
                .addr_humanize(&token_info.mint.unwrap().minter)
                .unwrap()
        );
    }
}
