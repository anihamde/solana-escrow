use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    #[error("Invalid Party")]
    InvalidParty,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    #[error("Own Escrow Deposit Incomplete")]
    OwnEscrowDepositIncomplete,
    #[error("Counterpary Escrow Deposit Incomplete")]
    CounterpartyEscrowDepositIncomplete,
    #[error("Already Deposited")]
    AlreadyDeposited,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
