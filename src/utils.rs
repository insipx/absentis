use std::fmt::LowerHex;

crate trait IntoHexStr {

    fn into_hex_str<T>(num: T) -> String where T: LowerHex;
}

impl IntoHexStr where {
    fn into_hex_str<usize>(&self) -> String {
        format!("{:#x}", self)
    }
}

