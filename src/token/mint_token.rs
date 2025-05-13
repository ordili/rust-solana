use crate::common;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, signature::{Keypair, Signer}, system_instruction::create_account, transaction::Transaction
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    id as token_2022_program_id,
    instruction::{initialize_mint, mint_to},
    state::Mint,
};

fn mint_token_example(client : &RpcClient) -> Result<()> {
    let recent_blockhash = client.get_latest_blockhash()?;

    // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();

    // Airdrop 1 SOL to fee payer
    let airdrop_signature = client.request_airdrop(&fee_payer.pubkey(), 1_000_000_000)?;
    client.confirm_transaction(&airdrop_signature)?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature)?;
        if confirmed {
            break;
        }
    }

    // Generate keypair to use as address of mint
    let mint = Keypair::new();

    // Get default mint account size (in bytes), no extensions enabled
    let mint_space = Mint::LEN;
    let mint_rent = client.get_minimum_balance_for_rent_exemption(mint_space)?;

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
    let associated_token_account = get_associated_token_address_with_program_id(
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
    let transaction_signature = client.send_and_confirm_transaction(&transaction)?;

    println!("Payer Address: {}", fee_payer.pubkey());
    println!("Mint Address: {}", mint.pubkey());
    println!(
        "Associated Token Account Address: {}",
        associated_token_account
    );
    println!("Transaction Signature: {}", transaction_signature);

    // Get the latest blockhash for the mint transaction
    let recent_blockhash = client.get_latest_blockhash()?;

    // Amount of tokens to mint (1.00 tokens with 2 decimals)
    let amount = 100;

    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        &mint.pubkey(),            // mint
        &associated_token_account, // destination
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
    let transaction_signature = client.send_and_confirm_transaction(&transaction)?;

    println!("Successfully minted {} tokens to the associated token account", amount);
    println!("Transaction Signature: {}", transaction_signature);

    Ok(())
}

#[cfg(test)]
mod tests {    
    use solana_sdk::commitment_config::CommitmentConfig;

    use super::*;

    #[test]
    fn test_mint_token_example() {
        let client = RpcClient::new_with_commitment(
            String::from("http://127.0.0.1:8899"),
            CommitmentConfig::confirmed(),
        );
        mint_token_example(&client).unwrap();
    }
}
