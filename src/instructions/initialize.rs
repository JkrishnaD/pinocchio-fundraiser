use bytemuck::{Pod, Zeroable};
use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_token::state::{Mint, TokenAccount};

use crate::{constants::MIN_AMOUNT_TO_RAISE, error::FundraiserErrors, state::Fundraiser};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitializeInstructionData {
    pub amount: [u8; 8],
    pub duration: [u8; 8],
}

pub fn process_initialize_fundraiser(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        system_program,
        token_program,
        _associated_token_program,
        _rent_sysvar @ ..,
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
    let (fundraiser_pda, bump) =
        pubkey::find_program_address(&[b"fundraiser", maker.key().as_ref()], &crate::ID);
    if fundraiser.key() != &fundraiser_pda {
        return Err(ProgramError::InvalidAccountData);
    }
    // Vault account validation
    let vault_data = TokenAccount::from_account_info(vault)?;
    assert_eq!(vault_data.owner(), &fundraiser_pda,);
    assert_eq!(vault_data.mint(), mint_to_raise.key(),);

    let seed_bump = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key().as_ref()),
        Seed::from(&seed_bump),
    ];
    let signer = Signer::from(&seeds);

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
    .invoke_signed(&[signer])?;

    // Initializing the Fundraiser account
    let fundraiser_state = Fundraiser::load_mut(fundraiser)?;
    fundraiser_state.maker = *maker.key();
    fundraiser_state.bump = [bump];
    fundraiser_state.current_amount = 0u64.to_le_bytes();
    fundraiser_state.duration = instruction_data.duration;
    fundraiser_state.mint_to_raise = *mint_to_raise.key();
    fundraiser_state.amount_to_raise = instruction_data.amount;
    fundraiser_state.time_started = Clock::get()?.unix_timestamp.to_le_bytes();

    // // Create the vault account
    // pinocchio_associated_token_account::instructions::Create {
    //     funding_account: maker,
    //     account: vault,
    //     wallet: fundraiser,
    //     mint: mint_to_raise,
    //     token_program: token_program,
    //     system_program: system_program,
    // }
    // .invoke()?;

    Ok(())
}
