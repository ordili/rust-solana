use rust_solana::common::{get_account, get_rpc_client};
use solana_sdk::sysvar;
#[tokio::main]
async fn main() {
    let client = get_rpc_client();
    let pub_key_id = sysvar::clock::ID;
    let account_info = get_account(&client, &pub_key_id).await.unwrap();
    println!("{:#?}", account_info);
}
