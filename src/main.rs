use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() {
    let pub_key_str = "Cw1Q5ugnmkqhkeGu9y9QaGi1b837HiZtMrXFfNimxYXe";
    let pub_key = Pubkey::from_str_const(pub_key_str);
    println!("pubkey: {:?}", pub_key);
}