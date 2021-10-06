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
            EscrowInstruction::Deposit { amount } => {
                msg!("Instruction: Deposit");
                Self::process_exchange(accounts, amount, program_id)
            }
            EscrowInstruction::Withdraw { amount } => {
                msg!("Instruction: Withdraw");
                Self::process_exchange(accounts, amount, program_id)
            }
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
        // token program
        let token_program = next_account_info(account_info_iter)?;
        // system program
        let system_program = next_account_info(account_info_iter)?;
        // rent program
        let rent_program = next_account_info(account_info_iter)?;

        // create x_vault
        let x_vault = Self::create_pda_vault([alice, x_mint, token_program, system_program, rent_program])

        // create y_vault
        let y_vault = Self::create_pda_vault([alice, y_mint, token_program, system_program, rent_program])

        // create escrow
        let escrow = Self::create_pda_escrow([alice, bob, x_vault, y_vault, system_program, rent_program], amount_a, amount_b)

        let rent = &Rent::from_account_info(rent_program)?;
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        Ok(())
    }

    fn create_pda_vault(
        accounts: &[AccountInfo],
    ) -> AccountInfo {
        let account_info_iter = &mut accounts.iter();
        let alice = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let rent_program = next_account_info(account_info_iter)?;

        let (pda, bump) = Pubkey::find_program_address(&[b"vault"], mint.key);
        let vault: Vault = Vault { mint: mint.key, amount: 0, bump: bump};

        let space = 41;
        solana_program::program::invoke(
            &system_instruction::create_account(
                alice.key, //from_pubkey
                pda, //to_pubkey
                required_lamports, //lamports
                space, //space
                token_program_info.key, // owner
            ),
            &[alice.clone(), pda.clone(), system_program.clone()],
        )?;

    }

    fn process_exchange(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let takers_sending_token_account = next_account_info(account_info_iter)?;

        let takers_token_to_receive_account = next_account_info(account_info_iter)?;

        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
        let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        if amount_expected_by_taker != pdas_temp_token_account_info.amount {
            return Err(EscrowError::ExpectedAmountMismatch.into());
        }

        let initializers_main_account = next_account_info(account_info_iter)?;
        let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

        if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_info.initializer_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_info.initializer_token_to_receive_account_pubkey
            != *initializers_token_to_receive_account.key
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key,
            takers_sending_token_account.key,
            initializers_token_to_receive_account.key,
            taker.key,
            &[&taker.key],
            escrow_info.expected_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                takers_sending_token_account.clone(),
                initializers_token_to_receive_account.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            takers_token_to_receive_account.key,
            &pda,
            &[&pda],
            pdas_temp_token_account_info.amount,
        )?;
        msg!("Calling the token program to transfer tokens to the taker...");
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_token_to_receive_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pdas_temp_token_account.key,
            initializers_main_account.key,
            &pda,
            &[&pda],
        )?;
        msg!("Calling the token program to close pda's temp account...");
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                pdas_temp_token_account.clone(),
                initializers_main_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        msg!("Closing the escrow account...");
        **initializers_main_account.lamports.borrow_mut() = initializers_main_account
            .lamports()
            .checked_add(escrow_account.lamports())
            .ok_or(EscrowError::AmountOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;
        *escrow_account.data.borrow_mut() = &mut [];

        Ok(())
    }
}
