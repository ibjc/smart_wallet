#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, 
};

use smartwallet::incrementer::{
    ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, StateResponse
};

use crate::state::{CONFIG, STATE, Config, State};
use terra_cosmwasm::TerraMsgWrapper;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response<TerraMsgWrapper>> {

    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
    };

    let state = State {
        total: Uint128::zero(),
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
        ExecuteMsg::ResetOwner { owner } => update_owner(deps, info, owner),
        ExecuteMsg::Increment {} => increment(deps, info), 
        ExecuteMsg::Reset{ total } => reset(deps, info, total),
    }
}




#[allow(clippy::too_many_arguments)]
pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> StdResult<Response<TerraMsgWrapper>> {
    let api = deps.api;
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.owner != api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("Unauthorized"));
    }

    config.owner = api.addr_canonicalize(&api.addr_validate(&owner)?.to_string())?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

#[allow(clippy::too_many_arguments)]
pub fn increment(
    deps: DepsMut,
    info: MessageInfo,
) -> StdResult<Response<TerraMsgWrapper>> {
    let api = deps.api;
    let mut state: State = STATE.load(deps.storage)?;

    state.total += Uint128::from(1u32);
    
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![("action", "increment")]))
}

#[allow(clippy::too_many_arguments)]
pub fn reset(
    deps: DepsMut,
    info: MessageInfo,
    total: Uint128,
) -> StdResult<Response<TerraMsgWrapper>> {
    let api = deps.api;
    let mut state: State = STATE.load(deps.storage)?;

    state.total = total;
    
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![("action", "reset")]))
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
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    })
  }
  
  pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total: state.total,
    })
  }


