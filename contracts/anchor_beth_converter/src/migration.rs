use crate::state::{store_config, Config, KEY_CONFIG};
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::ReadonlySingleton;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: CanonicalAddr,
    pub anchor_token_address: Option<CanonicalAddr>,
    pub wormhole_token_address: Option<CanonicalAddr>,
    pub anchor_decimals: u8,
    pub wormhole_decimals: u8,
}

fn read_legacy_config(storage: &dyn Storage) -> StdResult<LegacyConfig> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn migrate_config(storage: &mut dyn Storage) -> StdResult<()> {
    let legacy_config: LegacyConfig = read_legacy_config(storage)?;

    store_config(storage).save(&Config {
        owner: legacy_config.owner,
        anchor_token_address: legacy_config.anchor_token_address,
        wormhole_token_address: legacy_config.wormhole_token_address,
    })
}
