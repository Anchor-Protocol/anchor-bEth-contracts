use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Asset {
    pub asset_name: String,
    // address of the token in wormhole
    pub wormhole_token_address: String,
    // address of bAsset that is connected to anchor
    pub anchor_token_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Receive interface for send token.
    /// Mint anchor beth token
    /// Burn anchor beth token and return
    /// wormhole beth token
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<String>,
    },
    WhitelisteAsset {
        asset: Asset,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    ConvertWormholeToAnchor {},
    ConvertAnchorToWormhole {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    WhitelistedAsset {
        asset_name: String,
    },
    WhitelistedAssets {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAssetResponse {
    pub asset: Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAssetsResponse {
    pub assets: Vec<Asset>,
}
