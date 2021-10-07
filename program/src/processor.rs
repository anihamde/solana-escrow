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
        program_pack::{IsInitialized, Pack}
    },
    std::convert::TryInto,
    spl_token::{
        instruction::initialize_account,
        state::Account,
    },
};
use crate::{instruction::EscrowInstruction, state::Escrow};
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

        let x_seed = x_mint.key.as_ref(); 
        let y_seed = y_mint.key.as_ref();

        // create x_vault
        Self::create_pda_vault(accounts, program_id, &x_seed);

        // create y_vault
        Self::create_pda_vault(accounts, program_id, &y_seed);

        // create escrow
        Self::create_pda_escrow(accounts, program_id, amount_a, amount_b);

        Ok(())
    }

    fn create_pda_vault(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        vault_seed: &[u8],
    ) -> ProgramResult {
        // get accounts
        let account_info_iter = &mut accounts.iter();
        // alice
        let alice = next_account_info(account_info_iter)?;
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

        let mut vault = x_vault;
        let mut mint = x_mint;
        if vault_seed == y_mint.key.as_ref() {
            vault = y_vault;
            mint = y_mint;
        }

        // seeds
        let alice_seed = alice.key.as_ref();
        let bob_seed = bob.key.as_ref();
        let (_, bump) = Pubkey::find_program_address(&[vault_seed, alice_seed, bob_seed], program_id);
        let seeds_with_bump = &[vault_seed, alice_seed, bob_seed, &[bump]];

        // rent and space
        let space = Account::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(vault.lamports());
        invoke_signed(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                vault.key, //to_pubkey
                required_lamports, //lamports
                space.try_into().unwrap(), //space
                token_program.key, // owner
            ),
            &[alice.clone(), vault.clone(), system_program.clone()],
            &[seeds_with_bump],
        )?;
        msg!("Done with creating account");

        // initialize account from token
        invoke(
            &initialize_account(
                token_program.key,
                vault.key,
                mint.key,
                escrow.key,
            )?,
            &[vault.clone(), mint.clone(), escrow.clone(), rent_program.clone(), token_program.clone()],
        )?;
        msg!("Done with initializing");

        // write data to struct
        let mut vault_data = Account::unpack_from_slice(&vault.data.borrow_mut())?;
        vault_data.mint = *mint.key;
        vault_data.amount = 0;
        vault_data.owner = *escrow.key;
        msg!("Done writing data");
        vault_data.pack_into_slice(&mut vault.data.borrow_mut());

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
        let x_mint = next_account_info(account_info_iter)?;
        let y_mint = next_account_info(account_info_iter)?;
        // vaults
        let x_vault = next_account_info(account_info_iter)?;
        let y_vault = next_account_info(account_info_iter)?;
        // escrow 
        let escrow = next_account_info(account_info_iter)?;

        // token program
        let _ = next_account_info(account_info_iter)?;
        // system program
        let system_program = next_account_info(account_info_iter)?;
        // rent program
        let rent_program = next_account_info(account_info_iter)?;

        // seeds
        let alice_seed = alice.key.as_ref();
        let bob_seed = bob.key.as_ref();
        let x_seed= x_mint.key.as_ref();
        let y_seed = y_mint.key.as_ref();
        let (_, bump) = Pubkey::find_program_address(&[x_seed, y_seed, alice_seed, bob_seed], program_id);
        let seeds_with_bump = &[x_seed, y_seed, alice_seed, bob_seed, &[bump]];

        // rent and space
        let space = Escrow::LEN;
        let rent = &Rent::from_account_info(rent_program)?;
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(escrow.lamports());
        invoke_signed(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                escrow.key, //to_pubkey
                required_lamports, //lamports
                space.try_into().unwrap(), //space
                program_id, // owner
            ),
            &[alice.clone(), escrow.clone(), system_program.clone()],
            &[seeds_with_bump],
        )?;
        msg!("Done with creating account!!!!!!");

        // get bump, write data to struct
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
        let mut new_escrow = Escrow::try_from_slice(&escrow.data.borrow_mut())?;

        Ok(())
    }
}
