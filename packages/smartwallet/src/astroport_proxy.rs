use cosmwasm_std::{Uint128, Coin, Decimal, WasmMsg};
use cosmwasm_bignumber::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::Cw20ReceiveMsg;
use astroport::{
    asset::{Asset, AssetInfo},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig{owner: String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BalanceCheck {address: String},
    FabricateProvideMsg {
        pair_address: String, 
        funds: Vec<Coin>, 
        assets: [Asset; 2], 
        slippage_tolerance: Option<Decimal>
    },
    //FabricateWithdrawMsg {address: String, amount: Uint128},
    //FabricateStakeMsg {amount: Uint128},
    //FabricateUnstakeMsg {amount: Uint128},
    //FabricateVoteMsg{amount: Uint128},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
}

//TODO: cw_asset::AssetList?
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BalanceCheckResponse {
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawActionsResponse {
    pub actions: Vec<WasmMsg>,
}