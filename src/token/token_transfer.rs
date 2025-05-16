use anyhow::{Ok, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{id as token_2022_program_id, instruction::transfer_checked};

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
    use super::*;
    use crate::common;
    use crate::token::comm;
    use spl_token_2022::state::Account;

    #[tokio::test]
    async fn test_token_transfer() -> Result<()> {
        let client = common::get_rpc_client();
        let authority = common::get_local_key_pair().unwrap();
        let mint_pubkey = Pubkey::from_str_const(comm::MINT_PUBKEY);
        let source_ata = Pubkey::from_str_const(comm::SOURCE_ATA_PUBKEY);
        let dest_ata = Pubkey::from_str_const(comm::DEST_ATA_PUBKEY);

        let before_token = client.get_token_account_balance(&dest_ata).await?;
        println!("before_token {:?}", before_token);
        let transfer_amount = 1;

        token_transfer(
            &client,
            &authority,
            &mint_pubkey,
            &source_ata,
            &dest_ata,
            transfer_amount,
        )
        .await?;

        let dest_ata_account_data = client.get_account_data(&dest_ata).await?;
        let dest_ata_account = Account::unpack_from_slice(&dest_ata_account_data).unwrap();
        let bf_amount: u64 = before_token.amount.parse().unwrap();
        assert_eq!(dest_ata_account.amount, bf_amount + transfer_amount);
        Ok(())
    }
}
