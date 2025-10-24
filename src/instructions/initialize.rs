use bytemuck::{Pod, Zeroable};
use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    pubkey,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_token::state::{Mint, TokenAccount};

use crate::{constants::MIN_AMOUNT_TO_RAISE, error::FundraiserErrors, state::Fundraiser};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitializeInstructionData {
    amount: [u8; 8],
    duration: [u8; 1],
    bump: [u8; 1],
}
pub fn process_initialize_fundraiser(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Initializing The Fundraiser");

    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        system_program,
        token_program,
        _associated_token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let instruction_data = bytemuck::try_from_bytes::<InitializeInstructionData>(&data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Validating the fundraiser account
    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let fundraiser_pda = pubkey::create_program_address(
        &[
            b"fundraiser",
            maker.key().as_ref(),
            &[instruction_data.bump[0]],
        ],
        &crate::ID,
    )?;
    if fundraiser.key() != &fundraiser_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate the vault account
    let vault_account =
        TokenAccount::from_account_info(vault).map_err(|_| ProgramError::InvalidAccountData)?;
    if vault_account.owner() != fundraiser.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if vault_account.mint() != mint_to_raise.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let mint_state =
        Mint::from_account_info(mint_to_raise).map_err(|_| ProgramError::InvalidAccountData)?;

    // Validating the minimum decimals of the mint
    let amount = u64::from_le_bytes(instruction_data.amount);
    if amount < MIN_AMOUNT_TO_RAISE.pow(mint_state.decimals() as u32) {
        return Err(FundraiserErrors::InvalidAmount.into());
    }

    // Creating the Fundraiser account
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: fundraiser,
        space: Fundraiser::LEN as u64,
        lamports: Rent::get()?.minimum_balance(Fundraiser::LEN),
        owner: &crate::ID,
    }
    .invoke()?;

    // Initializing the Fundraiser account
    let fundraiser_state = Fundraiser::load(fundraiser)?;
    fundraiser_state.maker = *maker.key();
    fundraiser_state.bump = instruction_data.bump;
    fundraiser_state.current_amount = 0u64.to_le_bytes();
    fundraiser_state.duration = instruction_data.duration;
    fundraiser_state.mint_to_raise = *mint_to_raise.key();
    fundraiser_state.amount_to_raise = instruction_data.amount;
    fundraiser_state.time_started = Clock::get()?.unix_timestamp.to_le_bytes();

    Ok(())
}
