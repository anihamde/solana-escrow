use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct Vault {
    pub mint: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

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

