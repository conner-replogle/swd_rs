use std::error::Error as StdError;
use thiserror::Error;

use crate::constant::{DapResult, DapResponse};

#[derive(Error, Debug)]
pub enum SwdError<PinError: StdError + 'static> {
    #[error("Dap Response {0:?}")]
    DapResponse(DapResponse),
    #[error(transparent)]
    PinError(#[from] PinError),
}