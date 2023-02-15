use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub arbiter: Addr,
    pub burn: Addr,
    pub development: Addr,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Record {
    pub burn: Uint128,
    pub jackpot: Uint128,
    pub development: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Limit {
    pub limit: Uint128,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const RECORD_KEY: &str = "record";
pub const RECORD: Item<Record> = Item::new(RECORD_KEY);

pub const LIMIT_KEY: &str = "limit";
pub const LIMIT: Item<Limit> = Item::new(LIMIT_KEY);
