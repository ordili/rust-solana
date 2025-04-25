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
    instruction::{initialize_mint, mint_to, transfer_checked},
    state::Mint,
};
use crate::common;

async fn token_transfer_example() -> Result<()> {
    // Create connection to local validator
    let client = common::get_rpc_client();
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();
    // Generate a second keypair for the token recipient
    let recipient = Keypair::new();

    common::airdrop(&client,&fee_payer, 1_000_000_000).await?;

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
    let source_token_address = get_associated_token_address_with_program_id(
        &fee_payer.pubkey(),      // owner
        &mint.pubkey(),           // mint
        &token_2022_program_id(), // program_id
    );

    // Instruction to create associated token account for fee_payer
    let create_source_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(),      // funding address
        &fee_payer.pubkey(),      // wallet address
        &mint.pubkey(),           // mint address
        &token_2022_program_id(), // program id
    );

    // Calculate the associated token account address for recipient
    let destination_token_address = get_associated_token_address_with_program_id(
        &recipient.pubkey(),      // owner
        &mint.pubkey(),           // mint
        &token_2022_program_id(), // program_id
    );

    // Instruction to create associated token account for recipient
    let create_destination_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(),      // funding address
        &recipient.pubkey(),      // wallet address
        &mint.pubkey(),           // mint address
        &token_2022_program_id(), // program id
    );

    // Amount of tokens to mint (100 tokens with 2 decimal places)
    let amount = 100_00;

    // Create mint_to instruction to mint tokens to the source token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        &mint.pubkey(),         // mint
        &source_token_address,  // destination
        &fee_payer.pubkey(),    // authority
        &[&fee_payer.pubkey()], // signer
        amount,                 // amount
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account_instruction,
            initialize_mint_instruction,
            create_source_ata_instruction,
            create_destination_ata_instruction,
            mint_to_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Address: {}", mint.pubkey());
    println!("Source Token Account Address: {}", source_token_address);
    println!(
        "Destination Token Account Address: {}",
        destination_token_address
    );
    println!("Setup Transaction Signature: {}", transaction_signature);
    println!("Minted {} tokens to the source token account", amount);

    // Get the latest blockhash for the transfer transaction
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Amount of tokens to transfer (0.50 tokens with 2 decimals)
    let transfer_amount = 50;

    // Create transfer_checked instruction to send tokens from source to destination
    let transfer_instruction = transfer_checked(
        &token_2022_program_id(), // program id
        &source_token_address,    // source
        &mint.pubkey(),           // mint
        &destination_token_address,// destination
        &fee_payer.pubkey(),      // owner of source
        &[&fee_payer.pubkey()],   // signers
        transfer_amount,          // amount
        2,                        // decimals
    )?;

    // Create transaction for transferring tokens
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "Successfully transferred 0.50 tokens from sender to recipient"
    );
    println!("Transaction Signature: {}", transaction_signature);

    // Get token account balances to verify the transfer
    let source_token_account = client.get_token_account(&source_token_address).await?;
    let destination_token_account = client.get_token_account(&destination_token_address).await?;

    if let Some(source_account) = source_token_account {
        println!(
            "Source Token Account Balance: {} tokens",
            source_account.token_amount.amount
        );
    }

    if let Some(destination_account) = destination_token_account {
        println!(
            "Destination Token Account Balance: {} tokens",
            destination_account.token_amount.amount
        );
    }

    Ok(())
}

mod tests {
    use super::*;
    #[test]
    fn test_get_account() {
        let account_info = tokio_test::block_on(token_transfer_example());
        println!("{:#?}", account_info);
    }
}
