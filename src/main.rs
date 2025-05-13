use solana_sdk::pubkey::Pubkey;

fn main() {
    let pub_key_str = "Cw1Q5ugnmkqhkeGu9y9QaGi1b837HiZtMrXFfNimxYXe";
    let pub_key = Pubkey::from_str_const(pub_key_str);
    // println!("pubkey: {:?}", pub_key);
    let seeds: &[&[u8]] = &[b"helloWorld"];
    let (pda, bump) = Pubkey::find_program_address(seeds, &pub_key);
    println!("pda : {:?}", pda);
    println!("bump: {:?}", bump)
}
