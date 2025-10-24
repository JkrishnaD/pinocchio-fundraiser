use pinocchio::program_error::ProgramError;

pub enum FundraiserErrors {
    InvalidAmount = 0x0,
}

impl From<FundraiserErrors> for ProgramError {
    fn from(e: FundraiserErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl FundraiserErrors {
    pub fn description(&self) -> &'static str {
        match self {
            FundraiserErrors::InvalidAmount => "Invalid amount",
        }
    }
}
