use pinocchio::program_error::ProgramError;

pub enum FundraiserErrors {
    InvalidAmount = 0x0,
    ContributionTooShort = 0x1,
    ContributionTooLong = 0x2,
    FundraiserExpired = 0x3,
    InvalidContributor = 0x4,
    FundraiserGoalReached = 0x5,
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
            FundraiserErrors::ContributionTooShort => "Contribution too short",
            FundraiserErrors::ContributionTooLong => "Contribution too long",
            FundraiserErrors::FundraiserExpired => "Fundraiser expired",
            FundraiserErrors::InvalidContributor => "Invalid contributor",
            FundraiserErrors::FundraiserGoalReached => "Fundraiser goal reached",
        }
    }
}
