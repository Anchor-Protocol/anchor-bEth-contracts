use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, Api, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};

use cw20::TokenInfoResponse;
use terra_cosmwasm::TerraQueryWrapper;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = String::from(MOCK_CONTRACT_ADDR);
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(
        MockQuerier::new(&[(&contract_addr, contract_balance)]),
        MockApi::default(),
    );

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    // first one is anchor token decimals, the second one is wormhole token decimals
    decimals: (u8, u8),
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: _,
            }) => {
                if contract_addr == "wormhole_token0000" {
                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "wormhole_token".to_string(),
                        symbol: "WORM".to_string(),
                        decimals: self.decimals.1,
                        total_supply: Default::default(),
                    })))
                } else {
                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "anchor_token".to_string(),
                        symbol: "ANC".to_string(),
                        decimals: self.decimals.0,
                        total_supply: Default::default(),
                    })))
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<TerraQueryWrapper>, _api: A) -> Self {
        WasmMockQuerier {
            base,
            decimals: (6, 8),
        }
    }

    pub fn set_decimals(&mut self, anchor_decimals: u8, wormhole_decimals: u8) {
        self.decimals = (anchor_decimals, wormhole_decimals)
    }
}
