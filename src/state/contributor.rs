use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Contributor {
    pub contributor: [u8; 32],
    pub amount: [u8; 8],
    pub bump: [u8; 1],
}

impl Contributor {
    pub const LEN: usize = core::mem::size_of::<Contributor>();

    pub fn load(contributor_account: &AccountInfo) -> Result<&Self, ProgramError> {
        let data = unsafe { contributor_account.borrow_data_unchecked() };
        let contributor_state = bytemuck::try_from_bytes::<Contributor>(data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(contributor_state)
    }

    pub fn load_mut(contributor_account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { contributor_account.borrow_mut_data_unchecked() };
        let contributor_state = bytemuck::try_from_bytes_mut::<Contributor>(data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(contributor_state)
    }
}
