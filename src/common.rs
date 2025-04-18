use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{bs58, pubkey};

pub fn create_keypair() -> Keypair {
    let keypair = Keypair::new();
    let public_address = keypair.pubkey();
    println!("public address: {public_address}");
    keypair
}

pub fn restore_keypair_from_secret_bytes() -> Keypair {
    let keypair_bytes = [
        188, 216, 33, 82, 41, 164, 217, 226, 89, 215, 164, 19, 77, 210, 71, 105, 225, 168, 110, 35,
        186, 151, 23, 182, 172, 0, 108, 218, 212, 79, 94, 51, 36, 188, 202, 153, 62, 245, 160, 129,
        54, 76, 158, 97, 207, 116, 118, 161, 141, 10, 82, 191, 10, 76, 231, 88, 199, 109, 226, 70,
        36, 35, 44, 34,
    ];
    let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();
    println!("recovered address: {}", keypair.pubkey());
    keypair
}

pub fn restore_keypair_from_secret_base58() -> Keypair {
    let keypair_base58 =
        "4UzFMkVbk1q6ApxvDS8inUxg4cMBxCQRVXRx5msqQyktbi1QkJkt574Jda6BjZThSJi54CHfVoLFdVFX8XFn233L";
    let keypair_bytes = bs58::decode(keypair_base58).into_vec().unwrap();
    let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();
    println!("recovered address: {}", keypair.pubkey());
    keypair
}
pub fn validate_public_key() {
    // Lies on the ed25519 curve and thus have an associated private key
    // Suitable for users
    let on_curve_public_key = pubkey!("5oNDL3swdJJF1g9DzJiZ4ynHXgszjAEpUkxVYejchzrY");
    println!("is on curve: {}", on_curve_public_key.is_on_curve());

    // Not on the ed25519 curve and thus have no associated private key
    // Not Suitable for users
    let off_curve_public_key = pubkey!("4BJXYkfvg37zEmBbsacZjeQDpTNx91KppxFJxRqrz48e");
    println!("is off curve: {}", off_curve_public_key.is_on_curve());
}

pub fn get_rpc_client() -> RpcClient {
    let client = RpcClient::new_with_commitment(
        String::from("http://127.0.0.1:8899"),
        CommitmentConfig::confirmed(),
    );
    client
}

pub async fn airdrop(client: &RpcClient, keypair: &Keypair, lamport: u64) -> anyhow::Result<()> {
    let transaction_signature = client.request_airdrop(&keypair.pubkey(), lamport).await?;
    loop {
        if client.confirm_transaction(&transaction_signature).await? {
            break;
        }
    }
    Ok(())
}
