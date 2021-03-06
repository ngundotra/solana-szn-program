//! Errors when sending and decoding messages
use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
/// Errors that arise as sols move
pub enum Sol2SolError {
    /// When system program id is incorrect
    #[error("Wrong system program address provided")]
    IncorrectSystemProgramAddress,
    /// When instruction data cannot be deserialized
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    /// Sanity check error
    #[error("Invalid number of spots in state")]
    SolBoxInvalidNumSpots,
    /// Signer needs more SOL to complete data
    #[error("Not enough SOL to complete transaction")]
    InsufficientFunds,
    /// When account has incorrect data
    #[error("Invalid account data provided")]
    InvalidAccountData,
    /// When a sol box is out of space
    #[error("Sol box has no space left for new messages")]
    SolBoxNoSpaceLeft,
    /// User-space `Owner` must be person paying
    #[error("Payer must be owner")]
    OwnerMismatch,
    /// User-space `owner` must own Sol Box 
    #[error("Payer must be owner")]
    SolBoxUserOwnerMismatch,
    /// System-space `Owner` must be person paying
    #[error("Payer must be owner")]
    SolBoxSystemOwnerMismatch,
    /// Make sure instruction data matches account info 
    #[error("Sol box info does not match passed pubkey")]
    IncorrectSolBox,
}
impl From<Sol2SolError> for ProgramError {
    fn from(e: Sol2SolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for Sol2SolError {
    fn type_of() -> &'static str {
        "Sol2Sol Error"
    }
}