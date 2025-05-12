use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() {
    let client = rust_solana::common::get_rpc_client();
    let account = Keypair::new();
    rust_solana::create_data_account::create_data_account(&client,&account)
        .await
        .unwrap();
}

// 7:00 4 + HR BP
