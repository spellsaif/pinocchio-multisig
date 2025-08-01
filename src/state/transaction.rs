use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

// ------------------------------
// 🔐 Transaction State Definition
// ------------------------------

/// Represents a transaction proposed within a multisig system.
/// This holds information like approvals, execution status, target recipient, etc.
#[repr(C)] // Ensure memory layout matches what Solana expects (C-like memory layout)
pub struct Transaction {
    /// Unique index assigned to this transaction by the Multisig.
    pub index: u64,

    /// Public key of the proposer who initiated this transaction.
    pub proposer: Pubkey,

    /// Count of approvals this transaction has received.
    pub num_approvals: u8,

    /// Boolean flags indicating which members have approved.
    /// Maximum of 10 members are supported in this design.
    pub approvals: [bool; 10],

    /// Whether the transaction has already been executed (to prevent re-execution).
    pub executed: bool,

    /// Expiry timestamp after which the transaction is no longer valid.
    pub expiry: u64,

    /// Public key of the receiver account where lamports will be sent.
    pub receiver: Pubkey,

    /// Amount of lamports (1 SOL = 1_000_000_000 lamports) to send.
    pub lamports: u64,

    /// Bump seed used for PDA derivation of this transaction account.
    pub bump: u8,
}

// ------------------------------
// 🧠 Transaction Implementation
// ------------------------------

impl Transaction {
    /// Total size of the Transaction struct in bytes.
    /// Required to allocate the correct amount of space in Solana accounts.
    pub const LEN: usize =
        size_of::<u64>()          // index
        + size_of::<Pubkey>()     // proposer
        + size_of::<u8>()         // num_approvals
        + size_of::<[bool; 10]>() // approvals
        + size_of::<bool>()       // executed
        + size_of::<u64>()        // expiry
        + size_of::<Pubkey>()     // receiver
        + size_of::<u64>()        // lamports
        + size_of::<u8>();        // bump

    /// ⚠️ UNSAFE: Get a mutable reference to the Transaction struct from raw account data.
    /// Use this **only** if you're sure the account contains a valid Transaction.
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> &mut Self {
        unsafe {
            &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self)
        }
    }

    /// ✅ SAFE: Load the Transaction from an account only if the data length matches.
    /// Helps prevent memory violations or invalid casts.
    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account_info.data_len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData); // Guard against invalid account size
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    /// Called when a member wants to approve this transaction.
    ///
    /// - Takes the index of the member (0 to 9)
    /// - Checks bounds to ensure it's a valid member
    /// - Only increments approval count if this is their **first** approval
    pub fn approve(&mut self, member_index: usize) -> Result<(), ProgramError> {
        // Prevent out-of-bounds access
        if member_index >= self.approvals.len() {
            return Err(ProgramError::InvalidArgument);
        }

        // If the member hasn't approved yet, mark them as approved
        if !self.approvals[member_index] {
            self.approvals[member_index] = true;
            self.num_approvals += 1;
        }

        Ok(())
    }

    /// Check if the transaction has expired based on the current Unix timestamp.
    ///
    /// - Returns true if the transaction is **no longer valid**
    /// - Used before execution to reject stale transactions
    pub fn is_expired(&self, current_timestamp: u64) -> bool {
        current_timestamp > self.expiry
    }
}
