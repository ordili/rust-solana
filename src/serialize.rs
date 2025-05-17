use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeMap;

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

mod test {
    use crate::serialize::Primitive;
    use borsh::{BorshDeserialize, BorshSerialize};
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
}
