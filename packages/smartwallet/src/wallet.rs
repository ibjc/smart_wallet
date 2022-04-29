use cosmwasm_std::{Uint128, Addr, CosmosMsg, Empty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ideally later we can also fabricate the cw3 during init
/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub hot_wallets: Vec<HotWallet>,
    pub cw3_address: String,
    pub whitelisted_contracts: Vec<WhitelistedContract>,
}
*/

/// ideally later we can also fabricate the cw3 during init
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum InstantiateMsg {
    
    ExistingMultiSig {
        hot_wallets: Vec<HotWallet>,
        cw3_address: String,
        whitelisted_contracts: Vec<WhitelistedContract>,
    },
    SpawnMultiSig{
        hot_wallets: Vec<HotWallet>,
        whitelisted_contracts: Vec<WhitelistedContract>,
        max_voting_period_in_blocks: u64,
        required_weight: u64,
        multisig_voters: Vec<MultiSigVoter>,
        cw3_code_id: u64,
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw3InstantiateMsg{
    pub voters: Vec<MultiSigVoter>,
    pub required_weight: u64,
    pub max_voting_period: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MultiSigVoter{
    pub addr: String,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Duration{
    Height(u64),
    Time(u64),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    
    //hardwired hot msgs with internal u64 ids
    AnchorEarnDeposit {amount: Uint128}, // id=0
    BlunaClaim {}, //id=1
    FillUpGas {}, // no id check

    //hot wallet mgmt; consider making a vector later on with a label field
    RemoveHot {address: String},
    UpsertHot {hot_wallet: HotWallet},

    //whitelisted contract mgmt
    ReplaceContractWhitelist { whitelisted_contracts: Vec<WhitelistedContract> },

    //update multsig
    ReplaceMultisig {address: String},

    //generalized exec for multisig
    Execute {command: CosmosMsg<Empty>},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    HotWallet {address: String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotWallet {
    pub address: String,
    pub label: String,
    pub gas_cooldown: u64,
    pub gas_tank_max: Uint128,
    pub whitelisted_messages: Vec<u64>, //cooldown for these too?
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedContract {
    pub address: String,
    pub label: String,
    pub code_id: u64, //this may be overkill
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub hot_wallets: Vec<HotWallet>,
    pub cw3_address: Addr,
    pub whitelisted_contracts: Vec<WhitelistedContract>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotWalletStateResponse {
    pub address: String,
    pub gas_time_left: u64,
}