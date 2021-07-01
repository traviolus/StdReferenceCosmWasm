use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::State;
use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Relay { symbols: Vec<String>, rates: Vec<u64>, resolve_times: Vec<u64>, request_ids: Vec<u64> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetRefs {},
    GetReferenceData { base: String, quote: String },
}

pub type ConfigResponse = State;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RefDataResponse {
    pub rate: Uint128,
    pub last_update: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: Uint128,
    pub last_updated_quote: Uint128,
}
