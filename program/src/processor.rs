use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    std::convert::TryInto,
};
use crate::{instruction::EscrowInstruction, state::Escrow, state::Vault};
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

        let x_seed = x_mint.key.to_bytes(); 

        // create x_vault
        Self::create_pda_vault(accounts, program_id, &x_seed);

        // create y_vault
        //Self::create_pda_vault(accounts, program_id, b"y");

        // create escrow
        //Self::create_pda_escrow(accounts, program_id, amount_a, amount_b);

        Ok(())
    }

    fn create_pda_vault(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        seed: &[u8],
    ) -> ProgramResult {
        // get accounts
        let account_info_iter = &mut accounts.iter();
        // alice
        let alice = next_account_info(account_info_iter)?;
        // bob
        let _ = next_account_info(account_info_iter)?;
        // mints
        let x_mint = next_account_info(account_info_iter)?;
        let y_mint = next_account_info(account_info_iter)?;
        // vaults
        let x_vault = next_account_info(account_info_iter)?;
        let y_vault = next_account_info(account_info_iter)?;
        // escrow 
        let _ = next_account_info(account_info_iter)?;

        // token program
        let token_program = next_account_info(account_info_iter)?;
        // system program
        let system_program = next_account_info(account_info_iter)?;
        // rent program
        let rent_program = next_account_info(account_info_iter)?;

        let mut vault = x_vault;
        let mut mint = x_mint;
        if seed == &y_mint.key.to_bytes() {
            vault = y_vault;
            mint = y_mint;
        }

        let (_, bump) = Pubkey::find_program_address(&[seed], program_id);
        let seeds_with_bump = &[seed, &[bump]];

        // rent and space
        let space = Vault::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(vault.lamports());
        msg!("Got to invoke");
        invoke_signed(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                vault.key, //to_pubkey
                required_lamports, //lamports
                space.try_into().unwrap(), //space
                token_program.key, // owner
            ),
            &[alice.clone(), vault.clone(), token_program.clone()],
            &[seeds_with_bump],
        )?;
        msg!("Done with invoke");

        // initialize account from spl token cpi

        // write data to struct
        // let mut vault_data = Vault::try_from_slice(&vault.data.borrow_mut())?;
        // vault_data.mint = *mint.key;
        // vault_data.amount = 0;
        // vault_data.bump = bump;
        // vault_data.serialize(&mut *vault.data.borrow_mut())?;

        Ok(())
    }

    fn create_pda_escrow(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount_a: u64,
        amount_b: u64,
    ) -> ProgramResult {
        // get accounts
        let account_info_iter = &mut accounts.iter();
        // alice
        let alice = next_account_info(account_info_iter)?;
        // bob
        let bob = next_account_info(account_info_iter)?;
        // mints
        let _ = next_account_info(account_info_iter)?;
        let _ = next_account_info(account_info_iter)?;
        // vaults
        let x_vault = next_account_info(account_info_iter)?;
        let y_vault = next_account_info(account_info_iter)?;
        // escrow 
        let escrow = next_account_info(account_info_iter)?;

        // token program
        let _ = next_account_info(account_info_iter)?;
        // system program
        let _ = next_account_info(account_info_iter)?;
        // rent program
        let rent_program = next_account_info(account_info_iter)?;

        // rent and space
        let space = Escrow::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(escrow.lamports());

        invoke(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                escrow.key, //to_pubkey
                required_lamports, //lamports
                space.try_into().unwrap(), //space
                program_id, // owner
            ),
            &[alice.clone(), escrow.clone()],
        )?;

        // get bump, write data to struct
        let (_, bump) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let mut escrow_data = Escrow::try_from_slice(&escrow.data.borrow_mut())?;
        escrow_data.party_a = *alice.key;
        escrow_data.party_b = *bob.key;
        escrow_data.size_a = amount_a;
        escrow_data.size_b = amount_b;
        escrow_data.vault_x = *x_vault.key;
        escrow_data.vault_y = *y_vault.key;
        escrow_data.state = 0;
        escrow_data.bump = bump;
        escrow_data.serialize(&mut *escrow.data.borrow_mut())?;

        Ok(())
    }
}
