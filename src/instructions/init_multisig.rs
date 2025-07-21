use pinocchio::{
    account_info::AccountInfo, 
    program_error::ProgramError, 
    pubkey::{
        self, 
        Pubkey
    }, 
    sysvars::{
        rent::Rent, 
        Sysvar
    }, 
    ProgramResult
};
use pinocchio_log::log;

use crate::state::{multisig_config, Multisig};

pub fn process_init_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [creator, multisig,multisig_config,treasury, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };
    // Multisig PDA
    let bump = unsafe{ *(data.as_ptr() as *const u8) }.to_le_bytes();
    let seed = [(b"multisig"), creator.key().as_slice(), bump.as_ref()];
    let seeds = &seed[..];

    let pda = pubkey::checked_create_program_address(seeds, &crate::ID).unwrap(); //derive_address
    assert_eq!(&pda, multisig.key());

    // Multisig_config PDA
    let multisig_config_seed = [(b"multisig_config"), multisig.key().as_slice(), bump.as_ref()];
    let multisig_config_seeds = &multisig_config_seed[..];
    let pda_config = pubkey::checked_create_program_address(multisig_config_seeds, &crate::ID).unwrap(); //derive_address
    assert_eq!(&pda_config, multisig_config.key());

    // Treasury PDA
    let treasury_seed = [(b"treasury"), multisig.key().as_slice(), bump.as_ref()];
    let treasury_seeds = &treasury_seed[..];
    let pda_treasury = pubkey::checked_create_program_address(treasury_seeds, &crate::ID).unwrap(); //derive_address
    assert_eq!(&pda_treasury, treasury.key());



    if multisig.owner() != &crate::ID {
        log!("Creating Multisig Account");

        // Create Multisig Account
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: multisig,
            lamports: Rent::get()?.minimum_balance(Multisig::LEN),
            space: Multisig::LEN as u64,
            owner: &crate::ID,
        }.invoke()?;

        // Populate Multisig Account
        let multisig_account = Multisig::from_account_info(&multisig)?;
        multisig_account.creator = *creator.key();
        multisig_account.num_members = unsafe { *(data.as_ptr().add(1) as *const u8) };
        multisig_account.members = [Pubkey::default(); 10]; // Initialize with default Pubkeys
        match multisig_account.num_members {
            0..=10 => {
                for i in 0..multisig_account.num_members as usize {
                    let member_key = unsafe { *(data.as_ptr().add(2 + i * 32) as *const [u8; 32]) };
                    multisig_account.members[i] = member_key;
                }
            },
            _ => return Err(ProgramError::InvalidAccountData),
        }
        multisig_account.bump = unsafe{ *(data.as_ptr() as *const u8) };
        

        log!("members: {}", unsafe { *(data.as_ptr().add(1) as *const u8)});
    }
    else {
        return Err(ProgramError::AccountAlreadyInitialized)
    }
    
    Ok(())
}