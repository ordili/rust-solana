use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{id as token_2022_program_id, instruction::mint_to};

async fn mint_token(
    client: &RpcClient,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) -> Result<()> {
    // Get the latest blockhash for the mint transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_2022_program_id(),
        mint_pubkey,                 // mint
        account_pubkey,              // destination
        &mint_authority.pubkey(),    // authority
        &[&mint_authority.pubkey()], // signer
        amount,                      // amount
    )?;

    // Create transaction for minting tokens
    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_instruction],
        Some(&mint_authority.pubkey()),
        &[&mint_authority],
        recent_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "Successfully minted {} tokens to the associated token account {}",
        amount, account_pubkey
    );
    println!("Transaction Signature: {}", transaction_signature);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;
    use crate::token::comm;

    #[tokio::test]
    async fn test_mint_token() -> Result<()> {
        let client = common::get_rpc_client();
        let mint_pubkey = Pubkey::from_str_const(comm::MINT_PUBKEY);
        let mint_authority = common::get_local_key_pair().unwrap();

        let account_pubkey = Pubkey::from_str_const(comm::SOURCE_ATA_PUBKEY);
        let amount = 1000;
        mint_token(
            &client,
            &mint_pubkey,
            &account_pubkey,
            &mint_authority,
            amount,
        )
        .await?;
        Ok(())
    }
}
