use cw20::{Cw20Coin, MinterResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TokenInstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub reward_contract: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MigrateMsg {
    pub minter: String,
}
