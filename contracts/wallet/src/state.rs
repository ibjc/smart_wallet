use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128, Addr, Deps, Api, Order, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Map, Item, Bound};
use smartwallet::wallet::{HotWallet};

pub const HOT_WALLETS: Map<String, HotWalletState> = Map::new("hotwallets");
pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub hot_wallets: Vec<HotWallet>,
    pub cw3_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotWalletState {
    pub address: String,
    pub last_gas_fillup: u64,
}
