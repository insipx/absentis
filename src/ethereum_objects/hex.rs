//! A general, unsized, Hex type
use serde::de::{self, Deserializer, Deserialize, Visitor};
use hex::FromHex;
use ethereum_types::Address;
use std::ops::Deref;
pub struct Hex (Vec<u8>);

impl Deref for Hex {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}


impl From<Hex> for Address {
    fn from(hex: Hex) -> Address {
        Address::from(&*hex.0)
    }
}
impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {

        struct HexVisitor;

        impl<'de> Visitor<'de> for HexVisitor {
            type Value = Hex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Hex")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: de::Error {

                let rmv_pfx = | x: &str | return x.char_indices().skip_while(|(i, x)| *i < 2).map(|(_, x)| x).collect::<String>();
                let get_hex = | x: &str | return Vec::from_hex(x).map_err(|e| de::Error::custom(e));
               
                // use 'clean_0x' from ethereum_types
                if v.starts_with("0x") && v.len() % 2 == 0 {
                    Ok( Hex(get_hex(&rmv_pfx(v))?) )
                
                } else if v.starts_with("0x") && v.len() % 2 != 0 {
                    let no_pfx = rmv_pfx(v);
                    let make_even = format!("{}{}", "0", no_pfx);
                    Ok( Hex(get_hex(&make_even)?) )
                
                } else if v.len() % 2 != 0 {
                    let make_even = format!("{}{}", "0", v);
                    Ok( Hex(get_hex(&make_even)?) )
                
                } else {
                    Ok( Hex(get_hex(v)?) )
                }
            }
        }
        deserializer.deserialize_str(HexVisitor)
    }
}

impl std::fmt::Debug for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let hex_str: String = self.0.iter().map(|b|  format!("{:x}", b)).collect();
        write!(f, "0x{}", hex_str)
    }
}


