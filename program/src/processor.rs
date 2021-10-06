use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::Account as TokenAccount;

use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow, state::Vault};
use borsh::{BorshDeserialize, BorshSerialize};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount_a, amount_b } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount_a, amount_b, program_id)
            }
            // EscrowInstruction::Deposit { amount } => {
            //     msg!("Instruction: Deposit");
            //     Self::deposit(accounts, amount, program_id)
            // }
            // EscrowInstruction::Withdraw { amount } => {
            //     msg!("Instruction: Withdraw");
            //     Self::withdraw(accounts, amount, program_id)
            // }
        }
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount_a: u64,
        amount_b: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // alice
        let alice = next_account_info(account_info_iter)?;

        // make alice always the payer for rent
        if !alice.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // bob
        let bob = next_account_info(account_info_iter)?;
        // mints
        let x_mint = next_account_info(account_info_iter)?;
        let y_mint = next_account_info(account_info_iter)?;
        // vaults
        let x_vault = next_account_info(account_info_iter)?;
        let y_vault = next_account_info(account_info_iter)?;
        // escrow 
        let escrow = next_account_info(account_info_iter)?;

        // token program
        let token_program = next_account_info(account_info_iter)?;
        // system program
        let system_program = next_account_info(account_info_iter)?;
        // rent program
        let rent_program = next_account_info(account_info_iter)?;

        // create x_vault
        Self::create_pda_vault([x_vault, alice, x_mint, token_program, system_program, rent_program], program_id, b"x_vault")

        // create y_vault
        Self::create_pda_vault([y_vault, alice, y_mint, token_program, system_program, rent_program], program_id, b"y_vault")

        // create escrow
        Self::create_pda_escrow([escrow, alice, bob, x_vault, y_vault, system_program, rent_program], program_id, amount_a, amount_b)

        Ok(())
    }

    fn create_pda_vault(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        seed: &[u8]
    ) -> AccountInfo {
        // get accounts
        let account_info_iter = &mut accounts.iter();
        let vault = next_account_info(account_info_iter)?;
        let alice = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let rent_program = next_account_info(account_info_iter)?;

        // rent and space
        let space = Vault::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(vault.lamports());
        solana_program::program::invoke(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                vault.key, //to_pubkey
                required_lamports, //lamports
                space, //space
                token_program_info.key, // owner
            ),
            &[alice.clone(), vault.clone(), system_program.clone()],
        )?;

        // get bump, write data to struct
        let (_, bump) = Pubkey::find_program_address(seed, program_id);
        let mut vault_data = Vault::try_from_slice(&vault.data.borrow_mut())?;
        vault_data.mint = mint.key
        vault_data.amount = 0
        vault_data.bump = bump
        vault_data.serialize(&mut *vault.data.borrow_mut())?;

        Ok(())
    }

    fn create_pda_escrow(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount_a: u64,
        amount_b: u64
    ) -> AccountInfo {
        // get accounts
        let account_info_iter = &mut accounts.iter();
        let escrow = next_account_info(account_info_iter)?;
        let alice = next_account_info(account_info_iter)?;
        let bob = next_account_info(account_info_iter)?;
        let x_vault = next_account_info(account_info_iter)?;
        let y_vault = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let rent_program = next_account_info(account_info_iter)?;

        // rent and space
        let space = Escrow::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(escrow.lamports());

        solana_program::program::invoke(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                escrow.key, //to_pubkey
                required_lamports, //lamports
                space, //space
                program_id, // owner
            ),
            &[alice.clone(), escrow.clone()],
        )?;

        // get bump, write data to struct
        let (_, bump) = Pubkey::find_program_address(b"escrow", program_id);
        let mut escrow_data = Escrow::try_from_slice(&escrow.data.borrow_mut())?;
        escrow.party_a = alice.key;
        escrow.party_b = bob.key;
        escrow.size_a = amount_a;
        escrow.size_b = amount_b;
        escrow.vault_x = x_vault.key;
        escrow.vault_y = y_vault.key;
        escrow.state = 0;
        escrow.bump = bump;
        escrow_data.serialize(&mut *escrow.data.borrow_mut())?;

        Ok(())
    }
}
