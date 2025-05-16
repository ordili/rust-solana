use crate::common::airdrop;
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token_2022::{id as token_2022_program_id, instruction::initialize_mint, state::Mint};

/**
* Creating a data account for a custom program takes two steps:
* Invoke the System Program to create an account, then transfer ownership to the custom program
* Invoke the custom program, which now owns the account, to initialize the account data as defined
* by the program's instruction
*/
pub async fn create_data_account(client: &RpcClient, mint: &Keypair) -> Result<()> {
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();
    println!("fee_payer : {:?}", fee_payer.pubkey());
    airdrop(client, &fee_payer, 1_000_000_000).await?;

    // Generate keypair to use as address of mint
    println!("mint : {:?}", mint.pubkey());

    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    // Step 1. Create account instruction
    let create_account_instruction = create_account(
        &fee_payer.pubkey(),      // fee payer
        &mint.pubkey(),           // mint address
        rent,                     // rent
        space as u64,             // space
        &token_2022_program_id(), // program id
    );

    // Step 2. Initialize mint instruction
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
    println!("Transaction Signature: {}", transaction_signature);

    let account_info = client.get_account(&mint.pubkey()).await?;
    println!("{:#?}", account_info);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;
    #[tokio::test]
    async fn test_create_account_one() -> Result<()> {
        let client = common::get_rpc_client();
        let account = Keypair::new();
        create_data_account(&client, &account).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_account_two() {
        let client = crate::common::get_rpc_client();
        let account = Keypair::new();
        create_data_account(&client, &account).await.unwrap();
    }
}
