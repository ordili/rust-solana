use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::account_info::{AccountInfo, next_account_info};
use solana_sdk::clock::Clock;
use solana_sdk::entrypoint::{ProgramResult};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction::create_account;
use solana_sdk::sysvar::Sysvar;
use solana_sdk::sysvar::clock::ID as SYSVAR_CLOCK_ID;
use solana_sdk::transaction::Transaction;
use solana_sdk::{msg, pubkey};
use solana_client::nonblocking::rpc_client::RpcClient;

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

///Now we pass the clock's sysvar public address via the client
async fn client_call_process_instruction(
    client: &RpcClient,
    fee_payer: &Keypair,
    hello_account: &Keypair,
) -> anyhow::Result<()> {

    let program_id = pubkey!("77ezihTV6mTh2Uf3ggwbYF2NyGJJ5HHah1GrdowWJVD3");
    let account_space = 1; // because there exists just one boolean variable

    let rent_required = client
        .get_minimum_balance_for_rent_exemption(account_space)
        .await?;

    let create_hello_acc_ix = create_account(
        &fee_payer.pubkey(),
        &hello_account.pubkey(),
        rent_required,
        account_space as u64,
        &program_id,
    );

    let ix_data = vec![];
    let accounts = vec![
        AccountMeta::new(fee_payer.pubkey(), true),
        AccountMeta::new(hello_account.pubkey(), false),
        AccountMeta::new(SYSVAR_CLOCK_ID, false),
    ];

    let pass_clock_ix = Instruction::new_with_bytes(program_id, &ix_data, accounts);

    let mut transaction = Transaction::new_with_payer(
        &[create_hello_acc_ix, pass_clock_ix],
        Some(&fee_payer.pubkey()),
    );

    transaction.sign(
        &[&fee_payer, &hello_account],
        client.get_latest_blockhash().await?,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }
    Ok(())
}

///Accessing Clock directly inside an instruction
// Creating the same instruction, but without expecting the SYSVAR_CLOCK_PUBKEY from the client side.

// Accounts required
/// 1. [signer, writable] Payer
/// 2. [writable] Hello state account
pub fn process_instruction_2(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Payer account
    let _payer_account = next_account_info(accounts_iter)?;
    // Hello state account
    let hello_state_account = next_account_info(accounts_iter)?;

    // Getting clock directly
    let clock = Clock::get()?;

    let mut hello_state = HelloState::try_from_slice(&hello_state_account.data.borrow())?;
    hello_state.is_initialized = true;
    hello_state.serialize(&mut &mut hello_state_account.data.borrow_mut()[..])?;
    msg!("Account initialized :)");

    // Getting timestamp
    let current_timestamp = clock.unix_timestamp;
    msg!("Current Timestamp: {}", current_timestamp);

    Ok(())
}


///The client side instruction, now only needs to pass the state and payer accounts.
async fn client_call_process_instruction_2(
    client: &RpcClient,
    fee_payer: &Keypair,
    hello_account: &Keypair,
) -> anyhow::Result<()> {
    let program_id = pubkey!("4ZEdbCtb5UyCSiAMHV5eSHfyjq3QwbG3yXb6oHD7RYjk");

    let account_space = 1; // because there exists just one boolean variable

    let rent_required = client
        .get_minimum_balance_for_rent_exemption(account_space)
        .await?;

    let create_hello_acc_ix = create_account(
        &fee_payer.pubkey(),
        &hello_account.pubkey(),
        rent_required,
        account_space as u64,
        &program_id,
    );

    let ix_data = vec![];
    let accounts = vec![
        AccountMeta::new(fee_payer.pubkey(), true),
        AccountMeta::new(hello_account.pubkey(), false),
    ];

    let pass_clock_ix = Instruction::new_with_bytes(program_id, &ix_data, accounts);

    let mut transaction = Transaction::new_with_payer(
        &[create_hello_acc_ix, pass_clock_ix],
        Some(&fee_payer.pubkey()),
    );
    transaction.sign(
        &[&fee_payer, &hello_account],
        client.get_latest_blockhash().await?,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    Ok(())
}