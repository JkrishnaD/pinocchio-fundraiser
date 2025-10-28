pub mod contribute;
pub mod initialize;
pub mod checker;
pub mod refund;

pub use contribute::*;
pub use initialize::*;
pub use checker::*;
pub use refund::*;

pub enum FundraiserInstructions {
    Initialize = 0,
    Contribute = 1,
    Checker = 2,
    Refund = 3,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::Contribute),
            2 => Ok(FundraiserInstructions::Checker),
            3 => Ok(FundraiserInstructions::Refund),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
