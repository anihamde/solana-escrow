use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Vault {
    pub mint: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Escrow {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub size_a: u64,
    pub size_b: u64, 
    pub vault_x: Pubkey,
    pub vault_y: Pubkey,
    pub state: u8,
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 41;
}

impl Escrow {
    pub const LEN: usize = 146;
}