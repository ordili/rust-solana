use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    pubkey, signature::Keypair, signer::Signer,
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
