use cosmwasm_std::{Binary, Decimal, Uint128, Addr};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsClaimRewards {
    pub hot_wallet: String,
    pub cold_wallets: Vec<String>,
    pub threshold: usize,
    pub max_expiration: u64,
}