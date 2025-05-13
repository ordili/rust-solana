use crate::common;
use anyhow::{Ok, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
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

async fn create_mint_account(
    client: &RpcClient,
    fee_payer: &Keypair,
    mint: &Keypair,
) -> Result<()> {
    println!("----------------------begin create_mint_account------------------------------");
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

    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Setup Transaction Signature: {}", transaction_signature);
    println!("----------------------end create_mint_account------------------------------\n");
    Ok(())
}

async fn mint_to_ata(
    client: &RpcClient,
    mint: &Keypair,
    authority: &Keypair,
    ata: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("----------------------begin mint_to_ata------------------------------");
    // Get the latest blockhash for the mint transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        &mint.pubkey(),         // mint
        ata,                    // destination
        &authority.pubkey(),    // authority
        &[&authority.pubkey()], // signer
        amount,                 // amount
    )?;

    // Create transaction for minting tokens
    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_instruction],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!(
        "Successfully minted {} tokens to the associated token account",
        amount
    );
    println!("Transaction Signature: {}\n", transaction_signature);
    println!("----------------------end mint_to_ata------------------------------");
    Ok(())
}

async fn create_associated_token_address(
    client: &RpcClient,
    wallet: &Keypair,
    mint: &Keypair,
) -> Result<Pubkey> {
    println!(
        "----------------------begin create_associated_token_address------------------------------"
    );
    // Calculate the associated token account address for fee_payer
    let token_address: Pubkey = get_associated_token_address_with_program_id(
        &wallet.pubkey(),         // owner
        &mint.pubkey(),           // mint
        &token_2022_program_id(), // program_id
    );

    // Instruction to create associated token account for fee_payer
    let create_ata_instruction = create_associated_token_account(
        &wallet.pubkey(),         // funding address
        &wallet.pubkey(),         // wallet address
        &mint.pubkey(),           // mint address
        &token_2022_program_id(), // program id
    );

    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_instruction],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "create_associated_token_address transaction signature: {}",
        transaction_signature
    );
    println!(
        "----------------------end create_associated_token_address------------------------------\n"
    );
    Ok(token_address)
}

async fn token_transfer_example(
    client: &RpcClient,
    fee_payer: &Keypair,
    mint: &Keypair,
    from_ata: &Pubkey,
    ata_to: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("----------------------begin token_transfer_example------------------------------\n");
    // Get the latest blockhash for the transfer transaction
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transfer_checked instruction to send tokens from source to destination
    let transfer_instruction = transfer_checked(
        &token_2022_program_id(), // program id
        from_ata,                 // source
        &mint.pubkey(),           // mint
        ata_to,                   // destination
        &fee_payer.pubkey(),      // owner of source
        &[&fee_payer.pubkey()],   // signers
        amount,                   // amount
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
        "token_transfer_example transaction signature: {:?}",
        transaction_signature
    );
    println!("----------------------end token_transfer_example------------------------------\n");
    Ok(())
}

mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_create_mint_account() -> Result<()> {
        let client = common::get_rpc_client();
        let fee_payer = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &fee_payer, LAMPORTS_PER_SOL * 10).await?;
        create_mint_account(&client, &fee_payer, &mint).await?;
        println!("create mint accout : {:?}", &mint.pubkey());
        Ok(())
    }

    #[actix_rt::test]
    async fn test_create_associated_token_address() -> Result<()> {
        let client = common::get_rpc_client();
        let fee_payer = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &fee_payer, LAMPORTS_PER_SOL * 10).await?;
        create_mint_account(&client, &fee_payer, &mint).await?;
        println!("create mint accout : {:?}", &mint.pubkey());

        let associated_token_address =
            create_associated_token_address(&client, &fee_payer, &mint).await?;
        println!(
            "associated_token_address is {:?}",
            &associated_token_address
        );
        Ok(())
    }

    #[actix_rt::test]
    async fn test_mint_to_ata() -> Result<()> {
        let client = common::get_rpc_client();
        let fee_payer = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &fee_payer, LAMPORTS_PER_SOL * 10).await?;

        create_mint_account(&client, &fee_payer, &mint).await?;

        println!("mint accout is : {:?}", &mint.pubkey());

        let from_ata = create_associated_token_address(&client, &fee_payer, &mint).await?;
        println!("from_ata is {:?}", &from_ata);

        let mint_amount = 1000;
        mint_to_ata(&client, &mint, &fee_payer, &from_ata, mint_amount).await?;

        Ok(())
    }

    #[actix_rt::test]
    async fn test_token_transfer_example() -> Result<()> {
        let client = common::get_rpc_client();
        let fee_payer = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &fee_payer, LAMPORTS_PER_SOL * 10).await?;

        create_mint_account(&client, &fee_payer, &mint).await?;

        println!("mint accout is : {:?}", &mint.pubkey());

        let from_ata = create_associated_token_address(&client, &fee_payer, &mint).await?;
        println!("from_ata is {:?}", &from_ata);

        let mint_amount = 1000;
        mint_to_ata(&client, &mint, &fee_payer, &from_ata, mint_amount).await?;

        let to_wallet = Keypair::new();
        common::airdrop(&client, &to_wallet, LAMPORTS_PER_SOL * 2).await?;
        println!("to wallet is : {:?}", &to_wallet.pubkey());
        let to_ata = create_associated_token_address(&client, &to_wallet, &mint).await?;
        let to_mint_amount= 3000;
        mint_to_ata(&client, &mint, &fee_payer, &to_ata, to_mint_amount).await?;
        println!("to_ata is {:?}", &to_ata);

        let transfer_amount = 200;
        token_transfer_example(
            &client,
            &fee_payer,
            &mint,
            &from_ata,
            &to_ata,
            transfer_amount,
        )
        .await?;

        Ok(())
    }
}
