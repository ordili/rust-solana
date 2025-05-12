use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig, compute_budget, native_token::LAMPORTS_PER_SOL,
    signature::Keypair, signer::Signer, system_instruction::transfer, transaction::Transaction,
};
use spl_memo::build_memo;

pub async fn send_sol(
    client: &RpcClient,
    from_keypair: &Keypair,
    to_pub_key: &Pubkey,
    lamport: u64,
) -> anyhow::Result<()> {
    let transfer_ix = transfer(&from_keypair.pubkey(), to_pub_key, lamport);

    let mut transaction = Transaction::new_with_payer(&[transfer_ix], Some(&from_keypair.pubkey()));

    transaction.sign(&[&from_keypair], client.get_latest_blockhash().await?);

    match client.get_balance(to_pub_key).await {
        Ok(num) => {
            println!("Before transfer balance is {}", num)
        }
        Err(_) => {}
    }

    estimate_cu_used(&client, &transaction).await.unwrap();

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    match client.get_balance(to_pub_key).await {
        Ok(num) => {
            println!("after transfer balance is {}", num)
        }
        Err(_) => {}
    }

    Ok(())
}

async fn estimate_cu_used(client: &RpcClient, tx: &Transaction) -> anyhow::Result<u64> {
    let sim_res = client.simulate_transaction(tx).await?;

    let units_consumed = sim_res
        .value
        .units_consumed
        .expect("couldn't estimate CUs used");

    println!("Simulated units consumed: {}", units_consumed);

    Ok(units_consumed)
}

async fn add_memo_to_transaction(
    client: &RpcClient,
    signer_keypair: &Keypair,
    memo: &str,
) -> anyhow::Result<Transaction> {
    let memo_ix = build_memo(memo.as_bytes(), &[&signer_keypair.pubkey()]);
    let mut transaction = Transaction::new_with_payer(&[memo_ix], Some(&signer_keypair.pubkey()));
    transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await?);
    Ok(transaction)
}

async fn add_priority_fees_to_transaction(
    client: &RpcClient,
    signer_keypair: &Keypair,
    to_keypair: &Keypair,
    lamport: u64,
) -> anyhow::Result<Transaction> {
    let modify_cu_ix = compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let add_priority_fee_ix = compute_budget::ComputeBudgetInstruction::set_compute_unit_price(1);

    let transfer_ix = transfer(&signer_keypair.pubkey(), &to_keypair.pubkey(), lamport);

    let mut transaction = Transaction::new_with_payer(
        &[modify_cu_ix, add_priority_fee_ix, transfer_ix],
        Some(&signer_keypair.pubkey()),
    );
    transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await?);

    Ok(transaction)
}

#[cfg(test)]
mod test {
    use super::*;

    #[actix_rt::test]
    async fn test_create_account() {
        let client = crate::common::get_rpc_client();
        // create_data_account()
        let from = Keypair::new();
        let to = Keypair::new().pubkey();
        let lamports = LAMPORTS_PER_SOL * 10;
        // send_sol(&client, &from, &to, lamports).await.unwrap();
    }
}
