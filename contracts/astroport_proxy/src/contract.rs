#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr, BankMsg, WasmMsg, CosmosMsg, Coin, SubMsg, Reply, StdError, Decimal
};

use cosmwasm_bignumber::Uint256;

use smartwallet::astroport_proxy::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, BalanceCheckResponse, RawActionsResponse};

use crate::state::{CONFIG, Config, CustodyContractInfo as StateCustodyContractInfo};
use moneymarket::{
    market::{ExecuteMsg as AnchorExecuteMsg, Cw20HookMsg as AnchorHookMsg},
    overseer::ExecuteMsg as OverseerExecuteMsg,
    custody::{ExecuteMsg as CustodyExecuteMsg, Cw20HookMsg as CustodyHookMsg},
};
use basset::reward::{ExecuteMsg as BassetExecuteMsg};
use crate::error::ContractError;
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg};
use astroport::{
    pair::{QueryMsg as PairQueryMsg, ExecuteMsg as PairExecuteMsg, ReverseSimulationResponse, SimulationResponse, Cw20HookMsg as PairHookMsg},
    asset::{Asset, AssetInfo},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    CONFIG.save(deps.storage, &Config{
        owner: info.sender,
    })?;
    
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig{
            owner,
        } => execute_update_config(deps, env, info, owner),
    }
}



pub fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String
) -> Result<Response, ContractError>{

    //owner check
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender{
        return Err(ContractError::Unauthorized{});
    }

    config.owner = deps.api.addr_validate(&owner)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::BalanceCheck {address} => Ok(to_binary(&query_balance_check(deps, env, address)?)?),
        QueryMsg::FabricateProvideMsg {pair_address, funds, assets, slippage_tolerance} => Ok(to_binary(&query_fabricate_provide_msg(deps, env, pair_address, funds, assets, slippage_tolerance)?)?),

        //QueryMsg::FabricateWithdrawMsg {address, amount} => Ok(to_binary(&query_fabricate_withdraw_msg(deps, env, address)?)?),
        //QueryMsg::FabricateStakeMsg {amount} => Ok(to_binary(&query_fabricate_stake_msg(deps, env, address)?)?),
        //QueryMsg::FabricateUnstakeMsg {amount} => Ok(to_binary(&query_fabricate_unstake_msg(deps, env, address)?)?),
        //QueryMsg::FabricateVoteMsg {amount} => Ok(to_binary(&query_fabricate_vote_msg(deps, env, address)?)?),
        
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    todo!()
}
  

pub fn query_balance_check(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}

pub fn query_fabricate_provide_msg(deps: Deps, env: Env, pair_address: String, funds: Vec<Coin>, assets: [Asset; 2], slippage_tolerance: Option<Decimal> ) -> StdResult<RawActionsResponse> {
    return Ok(RawActionsResponse {
        actions: vec![WasmMsg::Execute {
            contract_addr: deps.api.addr_validate(&pair_address)?.into(),
            funds: funds,
            msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                assets: assets,
                slippage_tolerance: slippage_tolerance,
                auto_stake: None,
                receiver: None,
            })?,
        }],
    });
}

pub fn query_fabricate_withdraw_msg(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}

pub fn query_fabricate_stake_msg(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}

pub fn query_fabricate_unstake_msg(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}

pub fn query_fabricate_vote_msg(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}

