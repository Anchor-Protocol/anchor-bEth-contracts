use beth::converter::Asset;
use cosmwasm_std::{to_vec, Api, CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, Singleton};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";
// the identifier is asset_name <asset_name, <wormhole_token, anchor_token>>
pub static WORMHOLE_WHITELISTED_ASSETS: Map<&[u8], String> =
    Map::new("wormhole_whitelisted_assets");
pub static ANCHOR_WHITELISTED_ASSETS: Map<&[u8], String> = Map::new("anchor_whitelisted_assets");
pub static WHITELISTED_ASSETS: Map<&[u8], Asset> = Map::new("whitelisted_assets");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
}

/// HashMap<<wormhole_token, anchor_token>
pub fn store_asset_wormhole(
    storage: &mut dyn Storage,
    api: &dyn Api,
    asset: &Asset,
) -> StdResult<()> {
    let key = api.addr_canonicalize(&asset.wormhole_token_address)?;
    let value = asset.anchor_token_address.to_string();
    WORMHOLE_WHITELISTED_ASSETS.save(storage, key.as_slice(), &value)
}

/// HashMap<anchor_token, wormhole_token>
pub fn store_asset_anchor(
    storage: &mut dyn Storage,
    api: &dyn Api,
    asset: &Asset,
) -> StdResult<()> {
    let key = api.addr_canonicalize(&asset.anchor_token_address)?;
    let value = asset.wormhole_token_address.to_string();
    ANCHOR_WHITELISTED_ASSETS.save(storage, key.as_slice(), &value)
}

pub fn store_asset(storage: &mut dyn Storage, asset: &Asset) -> StdResult<()> {
    let key = to_vec(&asset.asset_name.clone())?;
    WHITELISTED_ASSETS.save(storage, key.as_slice(), asset)
}

pub fn store_config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
