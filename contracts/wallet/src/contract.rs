#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, attr, Binary, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, Addr, CanonicalAddr, BankMsg, WasmMsg, CosmosMsg, Coin
};

use smartwallet::wallet::{
    ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, HotWallet, HotWalletStateResponse, WhitelistedContract
};

use crate::state::{CONFIG, HOT_WALLETS, Config, HotWalletState};
use terra_cosmwasm::TerraMsgWrapper;
use std::cmp::{max, min};
use crate::tax_querier::{query_balance, deduct_tax};
use moneymarket::market::ExecuteMsg::DepositStable;
use basset::reward::ExecuteMsg::ClaimRewards;
use crate::error::ContractError;

pub const GAS_BUFFER: u64 = 100000000u64;
pub const ANCHOR_MARKET_CONTRACT: &str = "anchor_market";
pub const BLUNA_REWARD_CONTRACT: &str = "bluna_reward";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let config = Config {
        hot_wallets: msg.hot_wallets,
        cw3_address: deps.api.addr_validate(&msg.cw3_address)?,
        whitelisted_contracts: msg.whitelisted_contracts,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {

        //hot wallet actions
        ExecuteMsg::AnchorEarnDeposit {amount} => execute_anchor_earn_deposit(deps, env, info, amount), //id=0
        ExecuteMsg::BlunaClaim{} => execute_bluna_claim_rewards(deps, env, info), //id=1
        ExecuteMsg::FillUpGas{} => execute_fill_up_gas(deps, env, info), //any

        //hot wallet mgmt; consider making a vector later on with a label field
        ExecuteMsg::RemoveHot {address} => execute_remove_hot(deps, env, info, address),
        ExecuteMsg::UpsertHot {hot_wallet} => execute_upsert_hot(deps, env, info, hot_wallet),

        ExecuteMsg::ReplaceContractWhitelist { whitelisted_contracts } => execute_replace_contracts(deps, env, info, whitelisted_contracts),

        //update multsig
        ExecuteMsg::ReplaceMultisig {address} => execute_replace_multisig(deps, env, info, address),

        //generalized exec for multisig
        ExecuteMsg::Execute {command} => execute_command(deps, env, info, command),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_anchor_earn_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;
    let id = 0u64;

    let hot_wallet_config = config.hot_wallets.iter().find(|&x| x.address == info.sender.to_string());

    //hot wallet check
    if hot_wallet_config.is_none(){
        return Err(ContractError::Unauthorized{});
    }

    //hot wallet is enabled for this action
    if hot_wallet_config.unwrap().whitelisted_messages.iter().find(|&&x| x == id).is_none(){
        return Err(ContractError::UnauthorizedAction{});
    }

    let anchor_market_contract = config.whitelisted_contracts.iter().find(|&x| x.label == String::from(ANCHOR_MARKET_CONTRACT));

    //contract check
    if anchor_market_contract.is_none(){
        return Err(ContractError::ContractNotWhitelisted{});
    }

    let earn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: anchor_market_contract.unwrap().address.clone(),
        funds: vec![Coin{
            denom: String::from("uusd"),
            amount: amount,
        }],
        msg: to_binary(&DepositStable{})?,
    });

    Ok(Response::new().add_attributes(vec![("action", "anchor_earn_deposit")]).add_message(earn_msg))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_bluna_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;
    let id = 1u64;

    let hot_wallet_config = config.hot_wallets.iter().find(|&x| x.address == info.sender.to_string());

    //hot wallet check
    if hot_wallet_config.is_none(){
        return Err(ContractError::Unauthorized{});
    }

    //hot wallet is enabled for this action
    if hot_wallet_config.unwrap().whitelisted_messages.iter().find(|&&x| x == id).is_none(){
        return Err(ContractError::UnauthorizedAction{});
    }

    let bluna_reward_contract = config.whitelisted_contracts.iter().find(|&x| x.label == String::from(BLUNA_REWARD_CONTRACT));

    //contract check
    if bluna_reward_contract.is_none(){
        return Err(ContractError::ContractNotWhitelisted{});
    }

    let claim_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bluna_reward_contract.unwrap().address.clone(),
        funds: vec![],
        msg: to_binary(&ClaimRewards{recipient: None})?,
    });


    Ok(Response::new().add_attributes(vec![("action", "bluna_claim_rewards")]).add_message(claim_msg))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_fill_up_gas(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;

    let mut hot_wallet_config = config.hot_wallets.iter().find(|&x| x.address == info.sender.to_string());

    //hot wallet check
    if hot_wallet_config.is_none(){
        return Err(ContractError::Unauthorized{});
    }

    let mut hot_wallet_state = HOT_WALLETS
        .may_load(deps.storage, info.sender.to_string())?
        .unwrap_or(
            HotWalletState{
                address: info.sender.to_string(), 
                last_gas_fillup: env.block.time.seconds() - hot_wallet_config.unwrap().gas_cooldown - 10u64,
        });

    //cooldown check
    if hot_wallet_state.last_gas_fillup + hot_wallet_config.unwrap().gas_cooldown > env.block.time.seconds(){
        return Err(ContractError::GasCooldown{});
    }

    //figure out how much gas needed to fill hot wallet's tank
    let hot_wallet_gas_level = query_balance(deps.as_ref(), info.sender.to_string(), String::from("uusd")).unwrap();

    let hot_wallet_gas_need = max(Uint128::zero(), hot_wallet_config.unwrap().gas_tank_max - hot_wallet_gas_level);

    //sufficient smart_wallet uusd check
    if query_balance(deps.as_ref(), env.contract.address.to_string(), String::from("uusd")).unwrap() < hot_wallet_gas_need + Uint128::from(GAS_BUFFER) {
        return Err(ContractError::SmartWalletGas{});
    }

    let bank_msg = CosmosMsg::Bank(BankMsg::Send{
        to_address: info.sender.to_string(),
        amount: vec![deduct_tax(
            &deps.querier,
            Coin{
                denom: String::from("uusd"),
                amount: hot_wallet_gas_need,
            }
        )?]
    });

    hot_wallet_state.last_gas_fillup = env.block.time.seconds();

    HOT_WALLETS.save(deps.storage, info.sender.to_string(), &hot_wallet_state);

    Ok(Response::new().add_attributes(vec![("action", "fill_up_gas")]).add_message(bank_msg))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_remove_hot(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //multisig check
    if info.sender.to_string() != config.cw3_address{
        return Err(ContractError::Unauthorized{});
    }

    let hot_wallet_config = config.hot_wallets.iter().find(|&x| x.address == address);

    //check if valid hot address
    if hot_wallet_config.is_none(){
        return Err(ContractError::InvalidHotAddress{});
    }

    //remove from state
    HOT_WALLETS.remove(deps.storage, address.clone());

    //remove from config
    config.hot_wallets.retain(|x| x.address != address);

    CONFIG.save(deps.storage, &config);

    Ok(Response::new().add_attributes(vec![("action", "remove_hot")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_upsert_hot(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    hot_wallet: HotWallet,
) -> Result<Response, ContractError> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //multisig check
    if info.sender.to_string() != config.cw3_address{
        return Err(ContractError::Unauthorized{});
    }

    //check if valid hot address
    let address: Addr = deps.api.addr_validate(&hot_wallet.address)?;

    //remove from config
    config.hot_wallets.retain(|x| x.address != address);
    config.hot_wallets.push(hot_wallet);

    CONFIG.save(deps.storage, &config);

    Ok(Response::new().add_attributes(vec![("action", "upsert_hot")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_replace_contracts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelisted_contracts: Vec<WhitelistedContract>,
) -> Result<Response, ContractError> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //multisig check
    if info.sender.to_string() != config.cw3_address{
        return Err(ContractError::Unauthorized{});
    }

    config.whitelisted_contracts = whitelisted_contracts;

    CONFIG.save(deps.storage, &config);

    Ok(Response::new().add_attributes(vec![("action", "replace_contracts")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_replace_multisig(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //multisig check
    if info.sender.to_string() != config.cw3_address{
        return Err(ContractError::Unauthorized{});
    }

    config.cw3_address = deps.api.addr_validate(&address)?;

    CONFIG.save(deps.storage, &config);

    Ok(Response::new().add_attributes(vec![("action", "replace_multisig")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_command(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    command: CosmosMsg,
) -> Result<Response, ContractError> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //multisig check
    if info.sender.to_string() != config.cw3_address{
        return Err(ContractError::Unauthorized{});
    }

    Ok(Response::new().add_attributes(vec![("action", "execute_command")]).add_message(command))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::HotWallet {address} => Ok(to_binary(&query_hot_wallet_state(deps, env, address)?)?),
    }
}


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        hot_wallets: config.hot_wallets,
        cw3_address: config.cw3_address,
        whitelisted_contracts: config.whitelisted_contracts,
    })
  }
  

pub fn query_hot_wallet_state(deps: Deps, env: Env, address: String) -> StdResult<HotWalletStateResponse> {

    let hot_wallet: HotWalletState = HOT_WALLETS.may_load(deps.storage, address)?.unwrap_or(HotWalletState{address: String::from(""), last_gas_fillup: env.block.time.seconds()});

    //gas_time_left is janky; it's the num seconds since last fillup
    Ok(HotWalletStateResponse{
        address: hot_wallet.address,
        gas_time_left: max(env.block.time.seconds() - hot_wallet.last_gas_fillup, 0u64)
    })
  }
