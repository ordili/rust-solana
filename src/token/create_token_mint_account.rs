use crate::common;
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token_2022::{id as token_2022_program_id, instruction::initialize_mint, state::Mint};

async fn create_mint_account(
    client: &RpcClient,
    fee_payer: &Keypair,
    mint: &Keypair,
) -> Result<()> {
    let recent_blockhash = client.get_latest_blockhash().await?;
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;
    // Create account instruction
    let create_account_instruction = create_account(
        &fee_payer.pubkey(),      // fee payer
        &mint.pubkey(),           // mint address
        rent,                     // rent
        space as u64,             // space
        &token_2022_program_id(), // program id
    );

    // Initialize mint instruction
    let initialize_mint_instruction = initialize_mint(
        &token_2022_program_id(),
        &mint.pubkey(),            // mint address
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        9,                         // decimals
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Address: {}", mint.pubkey());
    println!("Mint Transaction Signature: {}", transaction_signature);

    Ok(())
}

mod tests {
    use super::*;
    #[tokio::test]
    async fn test_create_mint_account() -> Result<()> {
        let client = common::get_rpc_client();
        let fee_payer = common::get_local_key_pair().unwrap();
        let mint = Keypair::new();
        let account_info = create_mint_account(&client,&fee_payer,&mint).await?;
        println!("{:#?}", account_info);
        Ok(())
    }
}
