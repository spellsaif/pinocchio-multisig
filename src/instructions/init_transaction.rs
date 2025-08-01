use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::state::{Transaction};
use pinocchio_log::log;

pub fn process_init_transaction_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [proposer, multisig, transaction_account, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let bump = unsafe { *(data.as_ptr() as *const u8) };
    let index = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let lamports = unsafe { *(data.as_ptr().add(9) as *const u64) };
    let expiry = unsafe { *(data.as_ptr().add(17) as *const u64) };
    let target_key = unsafe { *(data.as_ptr().add(25) as *const Pubkey) };

    // Verify PDA
    let seed = [(b"transaction"), multisig.key().as_ref(), &index.to_le_bytes(), &[bump]];
    let seeds = &seed[..];
    let pda = pubkey::checked_create_program_address(seeds, &crate::ID)?;
    assert_eq!(&pda, transaction_account.key());

    if transaction_account.owner() != &crate::ID {
        log!("Creating Transaction Account");

        // Create Transaction account
        pinocchio_system::instructions::CreateAccount {
            from: proposer,
            to: transaction_account,
            lamports: Rent::get()?.minimum_balance(Transaction::LEN),
            space: Transaction::LEN as u64,
            owner: &crate::ID,
        }
        .invoke()?;

        // Populate Transaction account
        let tx = Transaction::from_account_info(transaction_account)?;
        tx.index = index;
        tx.proposer = *proposer.key();
        tx.num_approvals = 0;
        tx.approvals = [false; 10];
        tx.executed = false;
        tx.expiry = expiry;
        tx.receiver = target_key;
        tx.lamports = lamports;
        tx.bump = bump;

        log!("Initialized Transaction index: {}, lamports: {}", index, lamports);
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
