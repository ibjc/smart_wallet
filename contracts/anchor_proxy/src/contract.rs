#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr, BankMsg, WasmMsg, CosmosMsg, Coin, SubMsg, Reply, StdError
};

use smartwallet::anchor_proxy::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, BalanceCheckResponse, CustodyContractInfo, Cw20HookMsg};

use crate::state::{CONFIG, Config, CustodyContractInfo as StateCustodyContractInfo};
use moneymarket::market::ExecuteMsg::{DepositStable, RepayStable};
use basset::reward::ExecuteMsg::ClaimRewards;
use crate::error::ContractError;
use cw20::Cw20ReceiveMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let mut custody_contracts: Vec<StateCustodyContractInfo> = vec![];

    for contract in msg.custody_contracts{
        custody_contracts.push(StateCustodyContractInfo{
            address: deps.api.addr_validate(&contract.address)?,
            collateral_token: deps.api.addr_validate(&contract.collateral_token)?,
            label: contract.label,
        });
    }

    CONFIG.save(deps.storage, &Config{
        owner: info.sender,
        market_contract: deps.api.addr_validate(&msg.market_contract)?,
        overseer_contract: deps.api.addr_validate(&msg.overseer_contract)?,
        custody_contracts: custody_contracts,
        liquidation_contract: deps.api.addr_validate(&msg.liquidation_contract)?,
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

        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::UpdateConfig{
            owner,
            market_contract,
            overseer_contract,
            liquidation_contract,
        } => Ok(Response::new()),
        ExecuteMsg::UpsertCustodyContract{custody_contract: CustodyContractInfo} => Ok(Response::new()),

        ExecuteMsg::DepositStable {} => Ok(Response::new()),
        ExecuteMsg::ClaimRewards { to } => Ok(Response::new()),

        ExecuteMsg::LiquidateCollateral { borrower: String } => Ok(Response::new()),

        ExecuteMsg::WithdrawCollateral { amount: Uint256 } => Ok(Response::new()),

        ExecuteMsg::SubmitBid { collateral_token, premium_slot } => Ok(Response::new()),
        ExecuteMsg::RetractBid {bid_idx, amount} => Ok(Response::new()),
        ExecuteMsg::ActivateBids {collateral_token, bids_idx} => Ok(Response::new()),
        ExecuteMsg::ClaimLiquidations {collateral_token, bids_idx} => Ok(Response::new()),
    }
}


pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::RedeemStable {}) => {
            // aust check

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            //redeem_stable(deps, cw20_sender_addr, cw20_msg.amount.into())
            Ok(Response::new())
        },

        Ok(Cw20HookMsg::DepositCollateral {}) => {
            // anchor-whitelisted collateral check

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            //deposit_collateral(deps, cw20_sender_addr, cw20_msg.amount.into())
            Ok(Response::new())
        },

        _ => Err(ContractError::InvalidReceiveMsg {}),
    }
}




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::BalanceCheck {address} => Ok(to_binary(&query_balance_check(deps, env, address)?)?),
    }
}


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    todo!()
}
  

pub fn query_balance_check(deps: Deps, env: Env, address: String) -> StdResult<BalanceCheckResponse> {
    todo!()
}
