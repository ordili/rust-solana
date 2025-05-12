#[tokio::main]
async fn main() {

    let client = rust_solana::common::get_rpc_client();
    // create_data_account()
    rust_solana::create_data_account::create_data_account(&client).await.unwrap();
}

// 7:00 4 + HR BP
