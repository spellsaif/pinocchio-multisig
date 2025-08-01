use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};



//Transaction State
#[repr(C)]
pub struct Transaction {
    /// The index assigned by the multisig
    pub index: u64,
    /// The account that proposed this transaction
    pub proposer: Pubkey,
    /// Number of approvals
    pub num_approvals: u8,
    /// Approval flags(max 10 members)
    pub approvals: [bool; 10],
    ///Whether transaction already been executed or not
    pub executed: bool,
    ///Expiry timestamp (Unix)
    pub expiry: u64,
    /// Target Account (receiver)
    pub receiver: Pubkey,
    ///Amount of lamports to send
    pub lamports: u64,
    ///PDA bump for this transaction account
    pub bump: u8,

}


impl Transaction {
    pub const LEN: usize = 
        size_of::<u64>() +         // index
        size_of::<Pubkey>() +      // proposer
        size_of::<u8>() +          // num_approvals
        size_of::<[bool; 10]>() +  // approvals
        size_of::<bool>() +        // executed
        size_of::<u64>() +         // expiry
        size_of::<Pubkey>() +      // target
        size_of::<u64>() +         // lamports
        size_of::<u8>();           // bump


         pub fn from_account_info_unchecked(account_info: &AccountInfo) -> &mut Self {
        unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account_info.data_len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    pub fn approve(&mut self, member_index: usize) -> Result<(), ProgramError> {
        if member_index >= self.approvals.len() {
            return Err(ProgramError::InvalidArgument);
        }

        if !self.approvals[member_index] {
            self.approvals[member_index] = true;
            self.num_approvals += 1;
        }

        Ok(())
    }

    pub fn is_expired(&self, current_timestamp: u64) -> bool {
        current_timestamp > self.expiry
    }
}
