use crate::common;
use anyhow::Result;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    id as token_2022_program_id,
    instruction::{initialize_mint, mint_to},
    state::Mint,
};
async fn mint_token_example() -> Result<()> {
    // Create connection to local validator
    let client = common::get_rpc_client();
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();

    common::airdrop(&client, &fee_payer, 1_000_000_000);

    // Generate keypair to use as address of mint
    let mint = Keypair::new();

    // Get default mint account size (in bytes), no extensions enabled
    let mint_space = Mint::LEN;
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(mint_space)
        .await?;

    // Instruction to create new account for mint (token 2022 program)
    let create_account_instruction = create_account(
        &fee_payer.pubkey(),      // payer
        &mint.pubkey(),           // new account (mint)
        mint_rent,                // lamports
        mint_space as u64,        // space
        &token_2022_program_id(), // program id
    );

    // Instruction to initialize mint account data
    let initialize_mint_instruction = initialize_mint(
        &token_2022_program_id(),
        &mint.pubkey(),            // mint
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        2,                         // decimals
    )?;

    // Calculate the associated token account address for fee_payer
    let associated_token_address = get_associated_token_address_with_program_id(
        &fee_payer.pubkey(),      // owner
        &mint.pubkey(),           // mint
        &token_2022_program_id(), // program_id
    );

    // Instruction to create associated token account
    let create_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(),      // funding address
        &fee_payer.pubkey(),      // wallet address
        &mint.pubkey(),           // mint address
        &token_2022_program_id(), // program id
    );

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Address: {}", mint.pubkey());
    println!(
        "Associated Token Account Address: {}",
        associated_token_address
    );
    println!("Transaction Signature: {}", transaction_signature);

    // Get the latest blockhash for the mint transaction
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Amount of tokens to mint (100 tokens with 2 decimal places)
    let amount = 100;

    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        &mint.pubkey(),            // mint
        &associated_token_address, // destination
        &fee_payer.pubkey(),       // authority
        &[&fee_payer.pubkey()],    // signer
        amount,                    // amount
    )?;

    // Create transaction for minting tokens
    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Successfully minted 1.00 tokens to the associated token account");
    println!("Transaction Signature: {}", transaction_signature);

    Ok(())
}


mod tests {
    use super::*;
    #[test]
    fn test_get_account() {
        let account_info = tokio_test::block_on(mint_token_example());
        println!("{:#?}", account_info);
    }
}
