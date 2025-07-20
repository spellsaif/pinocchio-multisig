pub mod init_multisig;

pub use init_multisig::*;

use pinocchio::program_error::ProgramError;

pub enum MultisigInstructions {
    InitMultisig = 0,
}

impl TryFrom<&u8> for MultisigInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(MultisigInstructions::InitMultisig),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}