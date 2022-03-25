use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128, Addr, Deps, Api, Order, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

pub const STATE: Item<State> = Item::new("\u{0}\u{5}state");
pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub hot_wallet: Addr,
    pub cold_wallets: Vec<Addr>,
    pub cold_x: u64,
    pub cold_n: u64,
    pub max_expiration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub cold_running: u64,
    pub expiration: u64,
    pub cold_x: u64,
    pub cold_n: u64,
    pub cold_native_transfer: NativeTransfer,
    pub cold_wasm_execute: WasmExecute,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NativeTransfer{
    pub address: Addr,
    pub denom: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WasmExecute{
    pub address: Addr,
    pub message: Binary,
}