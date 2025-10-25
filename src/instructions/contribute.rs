use bytemuck::{Pod, Zeroable};
use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_token::{
    instructions::Transfer,
    state::{Mint, TokenAccount},
};

use crate::{
    constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER},
    error::FundraiserErrors,
    state::{Contributor, Fundraiser},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ContributeInstructions {
    pub amount: u64,
}

pub fn process_contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [
        contributor,
        mint_to_raise,
        fundraiser,
        contributor_account,
        contributor_ata,
        vault,
        token_program,
        system_program,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let instruction_data = bytemuck::try_from_bytes::<ContributeInstructions>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Validate the signer
    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    // Validate amount is not zero
    if instruction_data.amount == 0 {
        return Err(FundraiserErrors::InvalidAmount.into());
    }

    // Validate the Fundraiser account
    if fundraiser.owner() != &crate::ID {
        return Err(ProgramError::InvalidAccountOwner);
    }

    let fundraiser_state = Fundraiser::load(fundraiser)?;
    // Validating the mint
    if fundraiser_state.mint_to_raise != *mint_to_raise.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validating the vault owner
    let vault_state = TokenAccount::from_account_info(vault)?;
    if vault_state.owner() != fundraiser.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_state.mint() != mint_to_raise.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Converting the amount_to_raise from bytes to u64
    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    let current_amount = u64::from_le_bytes(fundraiser_state.current_amount);

    // Check if fundraiser already reached goal
    if current_amount >= amount_to_raise {
        return Err(FundraiserErrors::FundraiserGoalReached.into());
    }

    // Check if the amount to contribute is less than the maximum allowed contribution
    let max_contribution = (amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
    if instruction_data.amount > max_contribution {
        return Err(FundraiserErrors::ContributionTooLong.into());
    }

    // Check if the amount to contribute meets the minimum amount required
    let mint_state = Mint::from_account_info(mint_to_raise)?;
    let min_contribution = 10_u64.pow(mint_state.decimals() as u32);
    if instruction_data.amount < min_contribution {
        return Err(FundraiserErrors::ContributionTooShort.into());
    }

    // Checking if the Fundraiser is expired
    let duration_i64 = fundraiser_state.duration[0] as i64;
    let current_time = Clock::get()?.unix_timestamp;
    let time_started = i64::from_le_bytes(fundraiser_state.time_started);
    if duration_i64 <= current_time - time_started {
        return Err(FundraiserErrors::FundraiserExpired.into());
    }

    let contributor_seeds = [
        b"contributor",
        fundraiser.key().as_ref(),
        contributor.key().as_ref(),
    ];

    let (contributor_pda, bump) = pubkey::find_program_address(&contributor_seeds, &crate::ID);

    // Check if the contributor PDA is the same as the contributor account
    if contributor_pda != *contributor_account.key() {
        return Err(FundraiserErrors::InvalidContributor.into());
    }

    // Validating contributor token account
    let contributor_ata_state = TokenAccount::from_account_info(contributor_ata)?;
    if contributor_ata_state.owner() != contributor.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if contributor_ata_state.mint() != mint_to_raise.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check contributor has enough tokens
    if contributor_ata_state.amount() < instruction_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    if contributor_account.data_is_empty() {
        msg!("Creating the contributor account");

        let seed_bump = [bump];
        let signer_seeds = [
            Seed::from(b"contributor"),
            Seed::from(fundraiser.key().as_ref()),
            Seed::from(contributor.key().as_ref()),
            Seed::from(&seed_bump),
        ];
        let signer = Signer::from(&signer_seeds);

        pinocchio_system::instructions::CreateAccount {
            from: contributor,
            to: contributor_account,
            space: Contributor::LEN as u64,
            lamports: Rent::get()?.minimum_balance(Contributor::LEN),
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;

        let contributor_state = Contributor::load(contributor_account)?;
        contributor_state.amount = instruction_data.amount.to_le_bytes();
        contributor_state.contributor = *contributor.key();
        contributor_state.bump = [bump]
    } else {
        // Validate existing account owner
        if contributor_account.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let contributor_state = Contributor::load(contributor_account)?;
        let existing_amount = u64::from_le_bytes(contributor_state.amount);
        let new_amount = existing_amount
            .checked_add(instruction_data.amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        contributor_state.amount = new_amount.to_le_bytes();
    }

    let current_amount = u64::from_le_bytes(fundraiser_state.current_amount);
    if current_amount >= amount_to_raise {
        return Err(FundraiserErrors::FundraiserGoalReached.into());
    }

    Transfer {
        from: contributor_ata,
        authority: contributor,
        to: vault,
        amount: instruction_data.amount,
    }
    .invoke()?;

    let mut current_amount_u64 = u64::from_le_bytes(fundraiser_state.current_amount);

    current_amount_u64 = current_amount_u64
        .checked_add(instruction_data.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    fundraiser_state.current_amount = current_amount_u64.to_le_bytes();
    Ok(())
}
