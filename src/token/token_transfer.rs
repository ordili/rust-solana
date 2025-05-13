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
    authority: &Keypair,
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
        &authority.pubkey(),      // payer
        &mint.pubkey(),           // new account (mint)
        mint_rent,                // lamports
        mint_space as u64,        // space
        &token_2022_program_id(), // program id
    );

    // Instruction to initialize mint account data
    let initialize_mint_instruction = initialize_mint(
        &token_2022_program_id(),
        &mint.pubkey(),            // mint
        &authority.pubkey(),       // mint authority
        Some(&authority.pubkey()), // freeze authority
        2,                         // decimals
    )?;

    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&authority.pubkey()),
        &[&authority, &mint],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "Mint account transaction signature: {}",
        transaction_signature
    );
    println!("----------------------end create_mint_account------------------------------\n");
    Ok(())
}

async fn mint_to_ata(
    client: &RpcClient,
    mint_pubkey: &Pubkey,
    authority: &Keypair,
    account_pubkey: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("----------------------begin mint_to_ata------------------------------");
    // Get the latest blockhash for the mint transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        mint_pubkey,            // mint
        account_pubkey,         // destination
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

async fn create_ata(client: &RpcClient, wallet: &Keypair, mint_pubkey: &Pubkey) -> Result<Pubkey> {
    println!("----------------------begin create_ata------------------------------");
    // Calculate the associated token account address for fee_payer
    let token_address: Pubkey = get_associated_token_address_with_program_id(
        &wallet.pubkey(),         // owner
        mint_pubkey,              // mint
        &token_2022_program_id(), // program_id
    );

    // Instruction to create associated token account for fee_payer
    let create_ata_instruction = create_associated_token_account(
        &wallet.pubkey(),         // funding address
        &wallet.pubkey(),         // wallet address
        mint_pubkey,              // mint address
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
        "create_ata transaction signature: {}",
        transaction_signature
    );
    println!("----------------------end create_ata------------------------------\n");
    Ok(token_address)
}

async fn token_transfer(
    client: &RpcClient,
    authority: &Keypair,
    mint_pubkey: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("----------------------begin token_transfer------------------------------\n");
    // Get the latest blockhash for the transfer transaction
    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transfer_checked instruction to send tokens from source to destination
    let transfer_instruction = transfer_checked(
        &token_2022_program_id(), // program id
        source_pubkey,            // source
        mint_pubkey,              // mint
        destination_pubkey,       // destination
        &authority.pubkey(),      // owner of source
        &[&authority.pubkey()],   // signers
        amount,                   // amount
        2,                        // decimals
    )?;

    // Create transaction for transferring tokens
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "token_transfer_example transaction signature: {:?}",
        transaction_signature
    );
    println!("----------------------end token_transfer------------------------------\n");
    Ok(())
}

mod tests {
    use solana_sdk::program_option::COption;
    use spl_token_2022::state::{Account, AccountState};

    use super::*;

    #[actix_rt::test]
    async fn test_create_mint_account() -> Result<()> {
        let client = common::get_rpc_client();
        let authority = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &authority, LAMPORTS_PER_SOL * 10).await?;

        let before_balance = client.get_balance(&authority.pubkey()).await?;
        create_mint_account(&client, &authority, &mint).await?;
        let after_balance = client.get_balance(&authority.pubkey()).await?;

        println!(
            "beore balance is {}, after balance is {}, the differ is {}",
            before_balance,
            after_balance,
            (before_balance - after_balance)
        );

        let mint_balance = client.get_balance(&mint.pubkey()).await?;
        println!("mint_balance balance is {}", mint_balance);

        println!("create mint account : {:?}", &mint.pubkey());

        let mint_data = client.get_account_data(&mint.pubkey()).await?;
        let mint_account_data = Mint::unpack_from_slice(&mint_data).unwrap();

        assert_eq!(mint_account_data.is_initialized, true);
        assert_eq!(mint_account_data.decimals, 2);
        assert_eq!(mint_account_data.supply, 0);

        assert_eq!(
            mint_account_data.freeze_authority,
            COption::Some(authority.pubkey())
        );
        assert_eq!(
            mint_account_data.mint_authority,
            COption::Some(authority.pubkey())
        );

        let mint_account = client.get_account(&mint.pubkey()).await?;
        assert_eq!(mint_account.executable, false);
        assert_eq!(mint_account.owner, token_2022_program_id());

        println!("authority account : {:?}", &authority.pubkey());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_create_ata() -> Result<()> {
        let client = common::get_rpc_client();
        let authority = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &authority, LAMPORTS_PER_SOL * 10).await?;
        create_mint_account(&client, &authority, &mint).await?;
        println!("create mint accout : {:?}", &mint.pubkey());

        let wallet = Keypair::new();
        common::airdrop(&client, &wallet, LAMPORTS_PER_SOL * 3).await?;
        println!("wallet is : {:?}\n", wallet.pubkey());
        let ata = create_ata(&client, &wallet, &mint.pubkey()).await?;
        println!("ata is {:?}", &ata);

        let ata_data = client.get_account_data(&ata).await?;
        let ata_account_data = Account::unpack_from_slice(&ata_data).unwrap();
        println!("\nata account data is : {:?}", ata_account_data);

        assert_eq!(ata_account_data.amount, 0);
        assert_eq!(ata_account_data.mint, mint.pubkey());
        assert_eq!(ata_account_data.owner, wallet.pubkey());
        assert_eq!(ata_account_data.is_native, COption::None);
        assert_eq!(ata_account_data.state, AccountState::Initialized);

        let ata_account = client.get_account(&ata).await?;
        println!("\nata account is : {:?}", ata_account);
        assert_eq!(token_2022_program_id(), ata_account.owner);
        Ok(())
    }

    #[actix_rt::test]
    async fn test_mint_to_ata() -> Result<()> {
        let client = common::get_rpc_client();
        let authority = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &authority, LAMPORTS_PER_SOL * 10).await?;

        create_mint_account(&client, &authority, &mint).await?;
        println!("mint accout is : {:?}", &mint.pubkey());

        let wallet = Keypair::new();
        common::airdrop(&client, &wallet, LAMPORTS_PER_SOL * 2).await?;
        let ata = create_ata(&client, &wallet, &mint.pubkey()).await?;
        println!("ata is {:?}", &ata);

        let mint_amount = 1000;
        mint_to_ata(&client, &mint.pubkey(), &authority, &ata, mint_amount).await?;

        let ata_data = client.get_account_data(&ata).await?;
        let ata_data_account = Account::unpack_from_slice(&ata_data).unwrap();
        assert_eq!(ata_data_account.amount, mint_amount);
        println!("\n ata_data_account is : {:?}", ata_data_account);
        Ok(())
    }

    #[actix_rt::test]
    async fn test_token_transfer() -> Result<()> {
        let client = common::get_rpc_client();

        let authority = Keypair::new();
        let mint = Keypair::new();
        common::airdrop(&client, &authority, LAMPORTS_PER_SOL * 10).await?;

        create_mint_account(&client, &authority, &mint).await?;

        println!("authority account is : {:?}", &authority.pubkey());
        println!("mint account is : {:?}", &mint.pubkey());

        let source_wallet = Keypair::new();
        let dest_wallet = Keypair::new();
        common::airdrop(&client, &source_wallet, LAMPORTS_PER_SOL * 2).await?;
        common::airdrop(&client, &dest_wallet, LAMPORTS_PER_SOL * 3).await?;

        println!("source_wallet account is : {:?}", &source_wallet.pubkey());
        println!("dest_wallet account is : {:?}", &dest_wallet.pubkey());

        let source_ata = create_ata(&client, &source_wallet, &mint.pubkey()).await?;
        let dest_ata = create_ata(&client, &dest_wallet, &mint.pubkey()).await?;

        let mint_amount = 1000;
        mint_to_ata(
            &client,
            &mint.pubkey(),
            &authority,
            &source_ata,
            mint_amount,
        )
        .await?;
        mint_to_ata(&client, &mint.pubkey(), &authority, &dest_ata, mint_amount).await?;

        println!("source_ata account is : {:?}", &source_ata);
        println!("dest_ata account is : {:?}", &dest_ata);

        let transfer_amount = 200;
        token_transfer(
            &client,
            &source_wallet,
            &mint.pubkey(),
            &source_ata,
            &dest_ata,
            transfer_amount,
        )
        .await?;

        let source_ata_account_data = client.get_account_data(&source_ata).await?;
        let dest_ata_account_data = client.get_account_data(&dest_ata).await?;

        let source_ata_account = Account::unpack_from_slice(&source_ata_account_data).unwrap();
        let dest_ata_account = Account::unpack_from_slice(&dest_ata_account_data).unwrap();

        assert_eq!(source_ata_account.amount, mint_amount - transfer_amount);
        assert_eq!(dest_ata_account.amount, mint_amount + transfer_amount);

        Ok(())
    }
}
