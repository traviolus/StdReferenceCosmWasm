use schemars::JsonSchema;
use std::collections::HashMap;
use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use serde::{Deserialize, Serialize};
use vectorize;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RefData {
    pub rate: u64,
    pub resolve_time: u64,
    pub request_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    #[serde(with="vectorize")]
    pub refs: HashMap<String, RefData>,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
