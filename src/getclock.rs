use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::account_info::{AccountInfo, next_account_info};
use solana_sdk::clock::Clock;
use solana_sdk::entrypoint::{ProgramResult, entrypoint};
use solana_sdk::msg;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::Sysvar;

///How to get clock in a program
// Getting a clock (ie, the current time) can be done in two ways:
// 1. Passing SYSVAR_CLOCK_PUBKEY into an instruction
// 2. Accessing Clock directly inside an instruction.
//
// It is nice to know both the methods, because some legacy programs still expect
// the SYSVAR_CLOCK_PUBKEY as an account.

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HelloState {
    is_initialized: bool,
}

/// 1. Passing SYSVAR_CLOCK_PUBKEY into an instruction
///
// Accounts required
/// 1. [signer, writable] Payer
/// 2. [writable] Hello state account
/// 3. [] Clock sys var
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Payer account
    let _payer_account = next_account_info(accounts_iter)?;
    // Hello state account
    let hello_state_account = next_account_info(accounts_iter)?;
    // Clock sysvar
    let sysvar_clock_pubkey = next_account_info(accounts_iter)?;

    let mut hello_state = HelloState::try_from_slice(&hello_state_account.data.borrow())?;
    hello_state.is_initialized = true;
    hello_state.serialize(&mut &mut hello_state_account.data.borrow_mut()[..])?;
    msg!("Account initialized :)");

    // Type casting [AccountInfo] to [Clock]
    let clock = Clock::from_account_info(&sysvar_clock_pubkey)?;
    // Getting timestamp
    let current_timestamp = clock.unix_timestamp;
    msg!("Current Timestamp: {}", current_timestamp);

    Ok(())
}
