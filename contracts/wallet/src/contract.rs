#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, attr, Binary, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, Addr, CanonicalAddr, BankMsg, WasmMsg, CosmosMsg, Coin
};

use smartwallet::wallet::{
    ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, HotWallet, HotWalletStateResponse
};

use crate::state::{CONFIG, HOT_WALLETS, Config, HotWalletState};
use terra_cosmwasm::TerraMsgWrapper;
use std::cmp::{max, min};
use crate::tax_querier::deduct_tax;
use moneymarket::market::ExecuteMsg::DepositStable;
use basset::reward::ExecuteMsg::ClaimRewards;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config = Config {
        hot_wallets: msg.hot_wallets,
        cw3_address: deps.api.addr_validate(&msg.cw3_address)?,
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
) -> StdResult<Response<TerraMsgWrapper>> {
    match msg {

        //hot wallet actions
        ExecuteMsg::AnchorEarnDeposit {amount} => execute_anchor_earn_deposit(deps, env, info, amount), //id=0
        ExecuteMsg::BlunaClaim{} => execute_bluna_claim_rewards(deps, env, info), //id=1
        ExecuteMsg::FillUpGas{} => execute_fill_up_gas(deps, env, info), //any

        //hot wallet mgmt; consider making a vector later on with a label field
        ExecuteMsg::RemoveHot {address} => execute_remove_hot(deps, env, info, address),
        ExecuteMsg::UpsertHot {hot_wallet} => execute_upsert_hot(deps, env, info, hot_wallet),

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
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "anchor_earn_deposit")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_bluna_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "bluna_claim_rewards")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_fill_up_gas(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "fill_up_gas")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_remove_hot(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "remove_hot")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_upsert_hot(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    hot_wallet: HotWallet,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "upsert_hot")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_replace_multisig(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "replace_multisig")]))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_command(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    command: CosmosMsg,
) -> StdResult<Response<TerraMsgWrapper>> {

    Ok(Response::new().add_attributes(vec![("action", "execute_command")]))
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
    })
  }
  

pub fn query_hot_wallet_state(deps: Deps, env: Env, address: String) -> StdResult<HotWalletStateResponse> {

    let hot_wallet: HotWalletState = HOT_WALLETS.may_load(deps.storage, address)?.unwrap_or(HotWalletState{address: String::from(""), last_gas_fillup: env.block.time.seconds()});

    Ok(HotWalletStateResponse{
        address: hot_wallet.address,
        gas_time_left: max(env.block.time.seconds() - hot_wallet.last_gas_fillup, 0u64)
    })
  }
