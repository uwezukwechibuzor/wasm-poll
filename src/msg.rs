use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Poll;

//#[cw_serde]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin_address: String,
}

//#[cw_serde]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePoll {
        question: String,
    },
    Vote {
        question: String, // what question are we responding to?
        choice: String,   // "yes or "no
    },
}

//#[cw_serde]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPoll { question: String },
    GetConfig {}
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetPollResponse {
    pub poll: Option<Poll>,
}
