use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 8],
    pub bump: [u8; 1],
}

impl Fundraiser {
    pub const LEN: usize = core::mem::size_of::<Fundraiser>();

    pub fn load(fundraiser_account: &AccountInfo) -> Result<&Self, ProgramError> {
        let data = unsafe { fundraiser_account.borrow_data_unchecked() };
        let fundraiser_state = bytemuck::try_from_bytes::<Fundraiser>(data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(fundraiser_state)
    }

    pub fn load_mut(fundraiser_account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { fundraiser_account.borrow_mut_data_unchecked() };
        let fundraiser_state = bytemuck::try_from_bytes_mut::<Fundraiser>(data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(fundraiser_state)
    }
}
