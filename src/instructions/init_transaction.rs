use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar}, // For rent-exemption calculation
    ProgramResult, // Just a type alias for Result<(), ProgramError>
};

use crate::state::Transaction;

// Logging macro
use pinocchio_log::log;

/// Initializes a new Transaction PDA account with metadata like proposer, amount, expiry, etc.
pub fn process_init_transaction_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Destructure the account list safely.
    // `proposer`: Who is proposing the transaction.
    // `multisig`: The multisig account this transaction belongs to.
    // `transaction_account`: The new PDA that will store the Transaction state.
    // `_remaining`: captures any remaining accounts (ignored here).
    let [proposer, multisig, transaction_account, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys); // At least 3 accounts are mandatory
    };

    // Parse input data buffer `data` manually using unsafe pointer casting.
    // Layout must be known and consistent.
    
    // `bump` is the bump seed used to derive the transaction PDA.
    let bump = unsafe { *(data.as_ptr() as *const u8) };

    // `index` is the transaction index (often based on a counter in the multisig).
    let index = unsafe { *(data.as_ptr().add(1) as *const u64) };

    // `lamports` is the amount of SOL this transaction will send.
    let lamports = unsafe { *(data.as_ptr().add(9) as *const u64) };

    // `expiry` is the Unix timestamp after which the transaction is invalid.
    let expiry = unsafe { *(data.as_ptr().add(17) as *const u64) };

    // `target_key` is the address that will receive the lamports.
    let target_key = unsafe { *(data.as_ptr().add(25) as *const Pubkey) };

    // ---------------- PDA VALIDATION ----------------

    // Derive the expected PDA for the Transaction account using known seeds:
    // "transaction", multisig pubkey, transaction index, and bump
    let seed = [
        b"transaction",                     // Static seed to namespace
        multisig.key().as_ref(),           // This transaction belongs to this multisig
        &index.to_le_bytes(),              // Include index to make each transaction unique
        &[bump],                           // Bump to make PDA derivation succeed
    ];

    // Generate the PDA from the seed list and program ID
    let pda = pubkey::checked_create_program_address(&seed, &crate::ID)?;

    // Ensure that the passed `transaction_account` matches the derived PDA
    assert_eq!(&pda, transaction_account.key());

    // ---------------- ACCOUNT CREATION ----------------

    // If the transaction_account is not owned by our program,
    // it means we still need to create and initialize it.
    if transaction_account.owner() != &crate::ID {
        log!("Creating Transaction Account");

        // System Program creates the account with the right rent and size
        pinocchio_system::instructions::CreateAccount {
            from: proposer, // The proposer pays for rent
            to: transaction_account, // The PDA we're initializing
            lamports: Rent::get()?.minimum_balance(Transaction::LEN), // Rent-exempt balance
            space: Transaction::LEN as u64, // Space to allocate in bytes
            owner: &crate::ID, // Our program will own this account
        }
        .invoke()?; // Actually perform the account creation

        // ---------------- STATE POPULATION ----------------

        // Load the account as mutable and cast into Transaction struct
        let tx = Transaction::from_account_info(transaction_account)?;

        // Assign all fields manually
        tx.index = index;
        tx.proposer = *proposer.key(); // Copy by value
        tx.num_approvals = 0; // No one has approved yet
        tx.approvals = [false; 10]; // Reset all 10 approvals to false
        tx.executed = false; // Not executed yet
        tx.expiry = expiry; // Timestamp after which it's invalid
        tx.receiver = target_key; // Destination for the SOL
        tx.lamports = lamports; // Amount of lamports (1 SOL = 1_000_000_000 lamports)
        tx.bump = bump; // Save bump for future PDA validation

        log!("Initialized Transaction index: {}, lamports: {}", index, lamports);
    } else {
        // If it's already owned by us, it's already initialized — throw error
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // All good
    Ok(())
}
