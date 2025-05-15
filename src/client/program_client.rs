use anchor_client::{
    Client, Cluster, Program,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer, system_program,
    },
};
use anchor_lang::prelude::*;
use std::rc::Rc;

declare_program!(counter);
use counter::{accounts::Counter, client::accounts, client::args};

fn initialize_and_incremenet_account(program: &Program<Rc<Keypair>>, counter: &Keypair) -> anyhow::Result<()> {
    // Build and send instructions
    println!("\nSend transaction with initialize and increment instructions");
    let initialize_ix = program
        .request()
        .accounts(accounts::InitializeCounter {
            counter: counter.pubkey(),
            payer: program.payer(),
            system_program: system_program::ID,
        })
        .args(args::InitializeCounter)
        .instructions()?
        .remove(0);

    let increment_ix = program
        .request()
        .accounts(accounts::Increment {
            counter: counter.pubkey(),
        })
        .args(args::Increment)
        .instructions()?
        .remove(0);

    let signature = program
        .request()
        .instruction(initialize_ix)
        .instruction(increment_ix)
        .signer(&counter)
        .send()
        .unwrap();

    println!("   Transaction confirmed: {}", signature);

    println!("\nFetch counter account data");
    let counter_account: Counter = program.account::<Counter>(counter.pubkey()).unwrap();

    println!("   Counter value: {}", counter_account.count);

    Ok(())
}

fn increment_account(program: &Program<Rc<Keypair>>, counter: &Keypair) -> anyhow::Result<()> {
    // Build and send instructions
    println!("\nSend transaction with increment instructions");

    let increment_ix = program
        .request()
        .accounts(accounts::Increment {
            counter: counter.pubkey(),
        })
        .args(args::Increment)
        .instructions()?
        .remove(0);

    let signature = program
        .request()
        .instruction(increment_ix)
        .signer(&counter)
        .send()
        .unwrap();

    println!("   Transaction confirmed: {}", signature);

    println!("\nFetch counter account data");
    let counter_account: Counter = program.account::<Counter>(counter.pubkey()).unwrap();

    println!("   Counter value: {}", counter_account.count);

    Ok(())
}

fn initialize_counter(program: &Program<Rc<Keypair>>, counter: &Keypair) -> anyhow::Result<()> {
    // Build and send instructions
    println!("\nSend transaction with initialize instructions");
    let initialize_ix = program
        .request()
        .accounts(accounts::InitializeCounter {
            counter: counter.pubkey(),
            payer: program.payer(),
            system_program: system_program::ID,
        })
        .args(args::InitializeCounter)
        .instructions()?
        .remove(0);

    let signature = program
        .request()
        .instruction(initialize_ix)
        .signer(&counter)
        .send()
        .unwrap();

    println!("   Transaction confirmed: {}", signature);

    println!("\nFetch counter account data");
    let counter_account: Counter = program.account::<Counter>(counter.pubkey()).unwrap();

    println!("   Counter value: {}", counter_account.count);

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::common;

    use super::*;

    #[test]
    fn test_initialize_counter() {
        let keypair_path = "/home/gidon/.config/solana/id.json".to_string();
        let payer = common::get_key_pair_from_local_json(&keypair_path).unwrap();

        let counter = Keypair::new();
        println!("Generated Keypairs:");
        println!("   Payer: {}", payer.pubkey());
        println!("   Counter: {}", counter.pubkey());

        // Create program client
        let client = Client::new_with_options(
            Cluster::Localnet,
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );
        let program: Program<Rc<Keypair>> = client.program(counter::ID).unwrap();
        initialize_counter(&program, &counter).unwrap();
    }

    #[test]
    fn test_initialize_and_incremenet_account(){
        let keypair_path = "/home/gidon/.config/solana/id.json".to_string();
        let payer = common::get_key_pair_from_local_json(&keypair_path).unwrap();

        let counter = Keypair::new();
        println!("Generated Keypairs:");
        println!("   Payer: {}", payer.pubkey());
        println!("   Counter: {}", counter.pubkey());

        // Create program client
        let client = Client::new_with_options(
            Cluster::Localnet,
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );
        let program: Program<Rc<Keypair>> = client.program(counter::ID).unwrap();
        initialize_and_incremenet_account(&program, &counter).unwrap();
    }
}
