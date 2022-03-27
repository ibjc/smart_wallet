use cosmwasm_std::StdError;
use thiserror::Error;

/// only purpose of this is to accommodate the execute command message.
/// have no idea how to convert a cosmosmsg<empty> to a terramsgwrapper
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),


  #[error("unauthorized")]
  Unauthorized {},
}