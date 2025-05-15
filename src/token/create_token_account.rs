use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token_2022::{
    id as token_2022_program_id,
    instruction::initialize_account,
    state::Account,
};

async fn crate_token_account(
    client: &RpcClient,
    payer: &Keypair,
    owner: &Keypair,
    token_account: &Keypair,
    mint_address: &Pubkey,
) -> Result<Signature> {

    // Get token account size (in bytes)
    let token_account_space = Account::LEN;
    let token_account_rent = client
        .get_minimum_balance_for_rent_exemption(token_account_space)
        .await?;

    // Instruction to create new account for token account (token 2022 program)
    let create_token_account_instruction = create_account(
        &owner.pubkey(),            // payer
        &token_account.pubkey(),    // new account (token account)
        token_account_rent,         // lamports
        token_account_space as u64, // space
        &token_2022_program_id(),   // program id
    );

    // Instruction to initialize token account data
    let initialize_token_account_instruction = initialize_account(
        &token_2022_program_id(),
        &token_account.pubkey(), // account
        mint_address,    // mint
        &owner.pubkey(),         // owner
    )?;

    let recent_blockhash = client.get_latest_blockhash().await?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_token_account_instruction,
            initialize_token_account_instruction,
        ],
        Some(&payer.pubkey()),
        &[&payer, &token_account],
        recent_blockhash,
    );

     let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;
     Ok(transaction_signature)
}

mod tests {
    use std::str::FromStr;
    use crate::common;
    use super::*;

    const  MINT_PUBKEY: &str = "6R2DtucAYsCnJDjgxPaqSieXqZ6jtyMuNmSPDZFwqjeL";

    #[actix_rt::test]
    async fn test_crate_token_account() -> Result<()> {
        let client = common::get_rpc_client();
        let payer = common::get_local_key_pair().unwrap();
        let owner = Keypair::new();
        let mint: Pubkey = Pubkey::from_str(MINT_PUBKEY).unwrap();
        let token_account: Keypair = Keypair::new();

        println!("mint account : {:?}", &mint);
        println!("token account : {:?}", &token_account.pubkey());
        println!("owner account : {:?}", &owner.pubkey());
        
        let sig = crate_token_account(&client, &payer, &payer,&token_account, &mint).await?;
        println!("create token account sig : {:?}", &sig);
        Ok(())
    }
}
