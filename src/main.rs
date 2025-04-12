use rust_solana::common::{airdrop, create_keypair, get_rpc_client};
use rust_solana::transaction::send_sol;
use solana_sdk::native_token::LAMPORTS_PER_SOL;

#[tokio::main]
async fn main() {
    let client = get_rpc_client();

    let from_keypair = create_keypair();
    let to_keypair = create_keypair();
    let lamport = LAMPORTS_PER_SOL;

    airdrop(&client, &from_keypair, 10 * lamport).await.unwrap();

    send_sol(&client, &from_keypair, &to_keypair, lamport)
        .await
        .unwrap();
}
