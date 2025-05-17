use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeMap;

use arrayref::{array_ref, array_refs};
use solana_program::{program_option::COption, pubkey::Pubkey};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq)]
struct Primitive(
    u8,
    u16,
    u32,
    String,
    String,
    [u8; 5],
    BTreeMap<String, String>,
);

/// Emulate how COption is 'unpacked'
fn deser_option(data: &[u8]) -> COption<Pubkey> {
    // Map the data block
    let ain = array_ref![data, 0, 36];
    let (base, key) = array_refs![ain, 4, 32];
    // Get the SOME or NONE u32
    let nos = u32::from_le_bytes(*base);
    // Construct the COption accordingly
    let opt: COption<Pubkey> = if nos == 0 {
        COption::None
    } else {
        COption::Some(Pubkey::new_from_array(*key))
    };
    opt
}

mod test {
    use crate::serialize::{Primitive, deser_option};
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_sdk::pubkey::Pubkey;
    use std::collections::BTreeMap;

    #[test]
    fn test_primitive() {
        let prim = [
            255u8, 255, 255, 255, 255, 255, 255, 5, 0, 0, 0, 104, 101, 108, 108, 111, 5, 0, 0, 0,
            119, 111, 114, 108, 100, 1, 2, 3, 4, 5, 2, 0, 0, 0, 8, 0, 0, 0, 99, 111, 111, 107, 98,
            111, 111, 107, 6, 0, 0, 0, 114, 101, 99, 105, 112, 101, 6, 0, 0, 0, 114, 101, 99, 105,
            112, 101, 10, 0, 0, 0, 105, 110, 103, 114, 101, 100, 105, 101, 110, 116,
        ];

        let pri = Primitive::try_from_slice(&prim).unwrap();
        assert_eq!(255, pri.0);
    }
    #[test]
    fn test_serialize() {
        let mut map = BTreeMap::new();
        map.insert(String::from("test"), String::from("value"));
        let primitive = Primitive {
            0: 255,
            1: 65535,
            2: 4294967295,
            3: "hello".to_string(),
            4: "world".to_string(),
            5: [1, 2, 3, 4, 5],
            6: map,
        };
        let mut buffer: Vec<u8> = Vec::new();
        primitive.serialize(&mut buffer).unwrap();
        let origin_primitive = Primitive::try_from_slice(&buffer).unwrap();

        assert_eq!(origin_primitive, primitive);
    }
    #[test]
    fn test_deser_option() {
        // From Typescript with borsh'ing
        let copt = [
            1u8, 0, 0, 0, 135, 202, 71, 214, 68, 105, 98, 176, 211, 130, 105, 2, 55, 187, 86, 186,
            109, 176, 80, 208, 77, 100, 221, 101, 20, 203, 149, 166, 96, 171, 119, 35,
        ];
        // Emulate COption deserialization
        let coption = deser_option(&copt);
        if coption.is_some() {
            println!("{:?}", coption.expect("Uh-oh"));
        }
        // As a Borsh Struct
        #[derive(BorshDeserialize, BorshSerialize, Debug)]
        struct TOption(u32, [u8; 32]);
        let toption = TOption::try_from_slice(&copt).unwrap();
        let pkey = Pubkey::new_from_array(toption.1);
        println!("Some = {:?} Pubkey = {:?}", toption.0, pkey);
    }
}
