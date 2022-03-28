use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Map, Item};
use smartwallet::wallet::{HotWallet, WhitelistedContract};

pub const HOT_WALLETS: Map<String, HotWalletState> = Map::new("hotwallets");
pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub hot_wallets: Vec<HotWallet>,
    pub cw3_address: Addr,
    pub whitelisted_contracts: Vec<WhitelistedContract>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotWalletState {
    pub address: String,
    pub last_gas_fillup: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotWalletActionState {
    pub action_id: u64,
    pub last_execution: u64,
}