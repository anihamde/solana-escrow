use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    InitEscrow {
        amount_a: u64,
        amount_b: u64, 
    },
    Deposit {
        amount: u64,
    },
    Withdraw {
        amount: u64,
    },
}
