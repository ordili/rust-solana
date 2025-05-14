use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    extension::{
        ExtensionType,
        confidential_transfer::instruction::{PubkeyValidityProofData, configure_account},
    },
    instruction::reallocate,
    solana_zk_sdk::encryption::{auth_encryption::AeKey, elgamal::ElGamalKeypair},
};
use spl_token_client::{
    client::{ProgramRpcClient, ProgramRpcClientSendTransaction},
    spl_token_2022::id as token_2022_program_id,
    token::{self, ExtensionInitializationParams, Token},
};
use spl_token_confidential_transfer_proof_extraction::instruction::{ProofData, ProofLocation};
use std::sync::Arc;

async fn create_confidential_mint(
    rpc_client: Arc<RpcClient>,
    authority: Arc<Keypair>,
    mint: &Keypair,
) -> Result<()> {
    // Set up program client for Token client
    let program_client = ProgramRpcClient::new(rpc_client, ProgramRpcClientSendTransaction);

    // Number of decimals for the mint
    let decimals = 9;

    // Create a token client for the Token-2022 program
    // This provides high-level methods for token operations
    let token = Token::new(
        Arc::new(program_client),
        &token_2022_program_id(), // Use the Token-2022 program (newer version with extensions)
        &mint.pubkey(),           // Address of the new token mint
        Some(decimals),           // Number of decimal places
        authority.clone(),        // Fee payer for transactions (cloning Arc, not keypair)
    );

    // Create extension initialization parameters
    // The ConfidentialTransferMint extension enables confidential (private) transfers of tokens
    let extension_initialization_params =
        vec![ExtensionInitializationParams::ConfidentialTransferMint {
            authority: Some(authority.pubkey()), // Authority that can modify confidential transfer settings
            auto_approve_new_accounts: true,     // Automatically approve new confidential accounts
            auditor_elgamal_pubkey: None,        // Optional auditor ElGamal public key
        }];

    // Create and initialize the mint with the ConfidentialTransferMint extension
    // This sends a transaction to create the new token mint
    let transaction_signature = token
        .create_mint(
            &authority.pubkey(),             // Mint authority - can mint new tokens
            Some(&authority.pubkey()),       // Freeze authority - can freeze token accounts
            extension_initialization_params, // Add the ConfidentialTransferMint extension
            &[&mint],                        // Mint keypair needed as signer
        )
        .await?;

    // Print results for user verification
    println!("Mint Address: {}", mint.pubkey());
    println!("Authority Address : {}", authority.pubkey());
    println!("Transaction Signature: {}\n", transaction_signature);

    Ok(())
}

// Load the keypair from the default Solana CLI keypair path (~/.config/solana/id.json)
// This enables using the same wallet as the Solana CLI tools
fn load_keypair() -> Result<Keypair> {
    // Get the default keypair path
    let keypair_path = "/home/gidon/.config/solana/id.json".to_string();
    // Read the keypair file directly into bytes using serde_json
    // The keypair file is a JSON array of bytes
    let file = std::fs::File::open(&keypair_path)?;
    let keypair_bytes: Vec<u8> = serde_json::from_reader(file)?;

    // Create keypair from the loaded bytes
    // This converts the byte array into a keypair
    let keypair = Keypair::from_bytes(&keypair_bytes)?;

    Ok(keypair)
}

async fn create_confidential_token_account(
    rpc_client: Arc<RpcClient>,
    wallet: Arc<Keypair>,
    token_mint_address: &Pubkey,
) -> Result<Pubkey> {
    // ===== Create and configure token account for confidential transfers =====
    println!("\nCreate and configure token account for confidential transfers");

    // Get the associated token account address for the owner
    let token_account_pubkey = get_associated_token_address_with_program_id(
        &wallet.pubkey(),         // Token account owner
        token_mint_address,       // Mint
        &token_2022_program_id(), // Token program ID
    );
    println!("Token Account Address: {}", token_account_pubkey);

    // Step 1: Create the associated token account
    let create_associated_token_account_instruction = create_associated_token_account(
        &wallet.pubkey(),         // Funding account
        &wallet.pubkey(),         // Token account owner
        token_mint_address,       // Mint
        &token_2022_program_id(), // Token program ID
    );

    // Step 2: Reallocate the token account to include space for the ConfidentialTransferAccount extension
    let reallocate_instruction = reallocate(
        &token_2022_program_id(),                      // Token program ID
        &token_account_pubkey,                         // Token account
        &wallet.pubkey(),                              // Payer
        &wallet.pubkey(),                              // Token account owner
        &[&wallet.pubkey()],                           // Signers
        &[ExtensionType::ConfidentialTransferAccount], // Extension to reallocate space for
    )?;

    // Step 3: Generate the ElGamal keypair and AES key for token account
    let elgamal_keypair =
        ElGamalKeypair::new_from_signer(&wallet, &token_account_pubkey.to_bytes())
            .expect("Failed to create ElGamal keypair");
    let aes_key = AeKey::new_from_signer(&wallet, &token_account_pubkey.to_bytes())
        .expect("Failed to create AES key");

    // The maximum number of Deposit and Transfer instructions that can
    // credit pending_balance before the ApplyPendingBalance instruction is executed
    let maximum_pending_balance_credit_counter = 65536;

    // Initial token balance is 0
    let decryptable_balance = aes_key.encrypt(0);

    // Generate the proof data client-side
    let proof_data = PubkeyValidityProofData::new(&elgamal_keypair)
        .map_err(|_| anyhow::anyhow!("Failed to generate proof data"))?;

    // Indicate that proof is included in the same transaction
    let proof_location =
        ProofLocation::InstructionOffset(1.try_into()?, ProofData::InstructionData(&proof_data));

    // Step 4: Create instructions to configure the account for confidential transfers
    let configure_account_instructions = configure_account(
        &token_2022_program_id(),               // Program ID
        &token_account_pubkey,                  // Token account
        token_mint_address,                     // Mint
        &decryptable_balance.into(),            // Initial balance
        maximum_pending_balance_credit_counter, // Maximum pending balance credit counter
        &wallet.pubkey(),                       // Token Account Owner
        &[],                                    // Additional signers
        proof_location,                         // Proof location
    )?;

    // Combine all instructions
    let mut instructions = vec![
        create_associated_token_account_instruction,
        reallocate_instruction,
    ];
    instructions.extend(configure_account_instructions);

    // Create and send the transaction
    let recent_blockhash = rpc_client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    let transaction_signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await?;
    println!(
        "Create Token Account Transaction Signature: {}",
        transaction_signature
    );

    Ok(token_account_pubkey)
}

pub async fn confidential_mint_token_to_ata(
    rpc_client: Arc<RpcClient>,
    payer: Arc<Keypair>,
    mint: Keypair,
    token_account_pubkey: Pubkey,
) -> Result<()> {
    // Set up program client for Token client
    let program_client = ProgramRpcClient::new(rpc_client.clone(), ProgramRpcClientSendTransaction);
    let decimals = 9;

    // Create a token client for the Token-2022 program
    // This provides high-level methods for token operations
    let token = Token::new(
        Arc::new(program_client),
        &token_2022_program_id(), // Use the Token-2022 program (newer version with extensions)
        &mint.pubkey(),           // Address of the new token mint
        Some(decimals),           // Number of decimal places
        payer.clone(),            // Fee payer for transactions
    );

    // Mint some tokens to the newly created token account
    // This gives the account some tokens to work with
    let mint_signature = token
        .mint_to(
            &token_account_pubkey,            // Destination account
            &payer.pubkey(),                  // Mint authority
            100 * 10u64.pow(decimals as u32), // Amount (100 tokens with decimal precision)
            &[&payer],                        // Signers
        )
        .await?;

    println!("Mint Tokens Transaction Signature: {}", mint_signature);

    // Deposit the tokens to confidential state
    // This converts regular tokens to confidential tokens
    println!("Deposit tokens to confidential state pending balance");
    let deposit_signature = token
        .confidential_transfer_deposit(
            &token_account_pubkey,            // The token account
            &payer.pubkey(),                  // Authority (owner) of the account
            100 * 10u64.pow(decimals as u32), // Amount to deposit (100 tokens)
            decimals,                         // Decimals of the token
            &[&payer],                        // Signers (owner must sign)
        )
        .await?;

    println!(
        "Confidential Transfer Deposit Signature: {}",
        deposit_signature
    );

    Ok(())
}

pub mod test {
    use solana_sdk::native_token::LAMPORTS_PER_SOL;

    use crate::common;

    use super::*;

    #[actix_rt::test]
    pub async fn test_create_confidential_mint() -> Result<()> {
        let clent = common::get_rpc_client();
        let authority = Arc::new(load_keypair()?);
        let mint = Keypair::new();
        create_confidential_mint(Arc::new(clent), authority, &mint).await?;
        Ok(())
    }

    #[actix_rt::test]
    pub async fn test_create_confidential_token_account() -> Result<()> {
        let client = Arc::new(common::get_rpc_client());
        let authority = Arc::new(load_keypair()?);
        let mint = Keypair::new();

        create_confidential_mint(Arc::clone(&client), Arc::clone(&authority), &mint).await?;

        let wallet = Keypair::new();
        common::airdrop2(Arc::clone(&client), &wallet.pubkey(), LAMPORTS_PER_SOL * 3).await?;
        println!("wallet is : {:?}", &wallet.pubkey());
        let token_account =
            create_confidential_token_account(client, Arc::new(wallet), &mint.pubkey()).await?;

        println!("\ntoken account is : {:?}", token_account);

        Ok(())
    }

    #[actix_rt::test]
    pub async fn test_confidential_mint_token_to_ata() -> Result<()> {
        let client = Arc::new(common::get_rpc_client());
        let authority = Arc::new(load_keypair()?);
        let mint = Keypair::new();

        create_confidential_mint(Arc::clone(&client), Arc::clone(&authority), &mint).await?;

        let wallet = Arc::new(Keypair::new());
        common::airdrop2(Arc::clone(&client), &wallet.pubkey(), LAMPORTS_PER_SOL * 3).await?;
        println!("wallet is : {:?}", &wallet.pubkey());

        let token_account = create_confidential_token_account(
            Arc::clone(&client),
            Arc::clone(&wallet),
            &mint.pubkey(),
        )
        .await?;

        confidential_mint_token_to_ata(
            Arc::clone(&client),
            Arc::clone(&authority),
            mint,
            token_account,
        )
        .await?;

        Ok(())
    }
}
