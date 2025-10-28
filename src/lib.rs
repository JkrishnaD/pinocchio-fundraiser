#![allow(warnings)]
use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, pubkey::Pubkey};

use crate::instructions::FundraiserInstructions;

mod constants;
mod error;
mod instructions;
mod state;
mod tests;

entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("CG1q69YqagtgKi4G22pNM3WPYeqs1MEBe79qAZGU4FNc");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => {
            instructions::process_initialize_fundraiser(accounts, data)?;
        }
        FundraiserInstructions::Contribute => {
            instructions::process_contribute(accounts, data)?;
        }
        FundraiserInstructions::Checker => {
            instructions::process_checker(accounts, data)?;
        }
        FundraiserInstructions::Refund => {
            instructions::process_refund(accounts, data)?;
        }
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
