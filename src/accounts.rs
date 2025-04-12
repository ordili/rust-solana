use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account_info::{AccountInfo, next_account_info};
use solana_sdk::entrypoint::ProgramResult;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::program::invoke_signed;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    signature::Keypair, signer::Signer, system_instruction,
    system_instruction::create_account as create_account_ix,
    system_program::ID as SYSTEM_PROGRAM_ID, sysvar::rent::ID as SYSVAR_RENT_ID,
    transaction::Transaction,
};

//  using the System Program createAccount instruction to create an account
async fn create_account(
    client: &RpcClient,
    from_keypair: &Keypair,
    new_account_keypair: &Keypair,
    data_len: usize,
) -> anyhow::Result<()> {
    let rent_exemption_amount = client
        .get_minimum_balance_for_rent_exemption(data_len)
        .await?;

    let create_acc_ix = create_account_ix(
        &from_keypair.pubkey(),        // payer
        &new_account_keypair.pubkey(), // new account
        rent_exemption_amount,         // rent exemption fee
        data_len as u64,               // space reseved for new account
        &SYSTEM_PROGRAM_ID,            //assigned program address
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_acc_ix], Some(&from_keypair.pubkey()));
    transaction.sign(
        &[&from_keypair, &new_account_keypair],
        client.get_latest_blockhash().await?,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    Ok(())
}

async fn create_pda(
    client: &RpcClient,
    from_keypair: &Keypair,
    program_id: &Pubkey,
) -> anyhow::Result<()> {
    let (pda_address, bump) =
        Pubkey::find_program_address(&[&from_keypair.pubkey().to_bytes()], &program_id);

    let data_size = 0;

    let ix_data = vec![data_size, bump];
    let accounts = vec![
        AccountMeta::new(from_keypair.pubkey(), true),
        AccountMeta::new(pda_address, false),
        AccountMeta::new(SYSVAR_RENT_ID, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
    ];

    let create_pda_ix = Instruction::new_with_bytes(*program_id, &ix_data, accounts);

    let mut transaction =
        Transaction::new_with_payer(&[create_pda_ix], Some(&from_keypair.pubkey()));
    transaction.sign(&[&from_keypair], client.get_latest_blockhash().await?);

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    Ok(())
}

// Sign with a PDA's Account
//program transfer tokens from one account to another
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let pda_account_info = next_account_info(account_info_iter)?;
    let to_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;

    // pass bump seed for saving compute budget
    let bump_seed = instruction_data[0];

    invoke_signed(
        &system_instruction::transfer(
            &pda_account_info.key,
            &to_account_info.key,
            100_000_000, // 0.1 SOL
        ),
        &[
            pda_account_info.clone(),
            to_account_info.clone(),
            system_program_account_info.clone(),
        ],
        &[&[b"escrow", &[bump_seed]]],
    )?;

    Ok(())
}

pub async fn get_account_balance(client: &RpcClient, pubkey: &Pubkey) -> anyhow::Result<u64> {
    let balance = client.get_balance(&pubkey).await?;
    println!("{} SOL", balance / LAMPORTS_PER_SOL);
    Ok(balance)
}
