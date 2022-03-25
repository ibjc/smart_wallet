#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, attr, Binary, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, Addr, CanonicalAddr, BankMsg, WasmMsg, CosmosMsg, Coin
};

use smartwallet::wallet::{
    ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, StateResponse, NativeTransferResponse, WasmExecuteResponse,
};

use crate::state::{CONFIG, STATE, Config, State, NativeTransfer, WasmExecute};
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

    let cold_wallets: Vec<Addr> = msg.cold_wallets.iter().map(|w| deps.api.addr_validate(w).unwrap()).collect();

    let config = Config {
        hot_wallet: deps.api.addr_validate(&msg.hot_wallet)?,
        cold_wallets: cold_wallets,
        threshold: msg.threshold,
        max_expiration: msg.max_expiration,
    };

    let state = State {
        cold_running: 0u64,
        expiration: 064,
        cold_confirmers: vec![],
        cold_native_transfer: NativeTransfer{address: deps.api.addr_humanize(&CanonicalAddr::from(vec![]))?, denom: String::from(""), amount: Uint128::zero()},
        cold_wasm_execute: WasmExecute{address: deps.api.addr_humanize(&CanonicalAddr::from(vec![]))?, message: Binary::from(vec![])},
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

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

        ExecuteMsg::ColdWasmExecute { address, command, expiration } => cold_execute(deps, env, info, address, command, expiration),
        ExecuteMsg::AnchorEarnDeposit { amount } => anchor_earn_deposit(deps, env, info, amount), 
        ExecuteMsg::BlunaClaim{} => bluna_claim(deps, env, info),
        ExecuteMsg::ColdConfirm {} => cold_confirm(deps, env, info),
        ExecuteMsg::ChangeHotWallet{ address } => update_hot_wallet(deps, env, info, address),
        ExecuteMsg::ColdNativeTransfer { address, denom, amount, expiration } => cold_native(deps, env, info, address, denom, amount, expiration),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cold_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    denom: String,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    //check privs
    let cold_wallets: Vec<Addr> = config.cold_wallets;
    if !cold_wallets.contains(&info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    //check existing cold executed
    if state.cold_running > 0u64{
        return Err(StdError::generic_err("cold msg in progress"));
    }

    //update state
    state.cold_running = 1u64;
    state.expiration = env.block.time.seconds() + max(min(expiration.unwrap_or(config.max_expiration), config.max_expiration), 0);
    state.cold_native_transfer = NativeTransfer{address: deps.api.addr_validate(&address)?, denom: denom, amount: amount};
    state.cold_confirmers.push(info.sender);

    STATE.save(deps.storage, &state);

    Ok(Response::new().add_attributes(vec![("action", "cold_native")]))
}

#[allow(clippy::too_many_arguments)]
pub fn cold_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    command: Binary,
    expiration: Option<u64>,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    //check privs
    let cold_wallets: Vec<Addr> = config.cold_wallets;
    if !cold_wallets.contains(&info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    //check existing cold executed
    if state.cold_running > 0u64{
        return Err(StdError::generic_err("cold msg in progress"));
    }

    //update state
    state.cold_running = 2u64;
    state.expiration = env.block.time.seconds() + max(min(expiration.unwrap_or(config.max_expiration), config.max_expiration), 0);
    state.cold_wasm_execute = WasmExecute{address: deps.api.addr_validate(&address)?, message: command};
    state.cold_confirmers.push(info.sender);

    STATE.save(deps.storage, &state);

    Ok(Response::new().add_attributes(vec![("action", "cold_execute")]))
}

#[allow(clippy::too_many_arguments)]
pub fn anchor_earn_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config: Config = CONFIG.load(deps.storage)?;

    if (config.hot_wallet != info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    let contract_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from("terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s"),
        funds: vec![Coin{
            denom: String::from("uusd"),
            amount: amount,
        }],
        msg: to_binary(&DepositStable{})?,
    });

    Ok(Response::new().add_attributes(vec![("action", "update_config")]).add_message(contract_msg))
}


#[allow(clippy::too_many_arguments)]
pub fn bluna_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config: Config = CONFIG.load(deps.storage)?;

    if (config.hot_wallet != info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    Ok(Response::new()
        .add_attributes(vec![("action", "update_config")])
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from("terra17yap3mhph35pcwvhza38c2lkj7gzywzy05h7l0"),
        funds: vec![],
        msg: to_binary(&ClaimRewards{recipient: None})?,
    })))
}

#[allow(clippy::too_many_arguments)]
pub fn cold_confirm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    //check privs
    let cold_wallets: Vec<Addr> = config.cold_wallets;
    if !cold_wallets.contains(&info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    //check existing cold executed
    if state.cold_running <= 0u64{
        return Err(StdError::generic_err("cold msg not in progress; nothing to confirm"));
    }

    //check if expired
    if state.expiration < env.block.time.seconds(){
        state.cold_running = 0u64;
        state.cold_confirmers = vec![];
        state.expiration = 0u64;
        STATE.save(deps.storage, &state);
        return Err(StdError::generic_err("cold msg expired; reverting"));
    }

    //check if already confirmed
    if state.cold_confirmers.contains(&info.sender){
        return Err(StdError::generic_err("already confirmed"));
    }

    state.cold_confirmers.push(info.sender);

    //TODO: need to persist list of confirmers

    let mut messages = vec![];

    //xth confirm kicks off the tx
    if state.cold_confirmers.len() >= config.threshold{
        if state.cold_running == 1u64 {
            let bank_msg: CosmosMsg<TerraMsgWrapper> = CosmosMsg::Bank(BankMsg::Send {
                to_address: state.cold_native_transfer.address.to_string(),
                amount: vec![deduct_tax(
                    &deps.querier,
                    Coin {
                        denom: state.cold_native_transfer.denom.to_string(),
                        amount: state.cold_native_transfer.amount.into(),
                    },
                )?],
            });

            messages.push(bank_msg);
        } else if state.cold_running == 2u64{
            
            //wasm message 
            let contract_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.cold_wasm_execute.address.to_string(),
                funds: vec![],
                msg: to_binary(&state.cold_wasm_execute.message)?,
            });
        
            messages.push(contract_msg);
        }

        state.cold_confirmers = vec![];
        state.expiration = 0u64;
        state.cold_running = 0u64;
    }

    STATE.save(deps.storage, &state);

    let res = Response::new()
    .add_attributes(vec![
        attr("action", "cold_confirm"),
    ])
    .add_messages(messages);

    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn update_hot_wallet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    hot_wallet: String
) -> StdResult<Response<TerraMsgWrapper>> {

    let mut config: Config = CONFIG.load(deps.storage)?;

    //check privs
    if !config.cold_wallets.contains(&info.sender){
        return Err(StdError::generic_err("Unauthorized"));
    }

    config.hot_wallet = deps.api.addr_validate(&hot_wallet)?;

    CONFIG.save(deps.storage, &config);

    Ok(Response::new().add_attributes(vec![("action", "changed hot wallet")]))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&query_state(deps)?)?),
    }
}


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        hot_wallet: (&config).hot_wallet.to_string(),
        cold_wallets: config.cold_wallets,
        threshold: config.threshold,
        max_expiration: config.max_expiration,
    })
  }
  
  pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = STATE.load(deps.storage)?;
    Ok(StateResponse {
        cold_running: state.cold_running,
        expiration: state.expiration,
        cold_confirmers: state.cold_confirmers,
        cold_native_transfer: NativeTransferResponse{address: state.cold_native_transfer.address, denom: state.cold_native_transfer.denom, amount: state.cold_native_transfer.amount},
        cold_wasm_execute: WasmExecuteResponse{address: state.cold_wasm_execute.address, message: state.cold_wasm_execute.message}
    })
  }