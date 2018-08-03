use std::fmt::LowerHex;

crate trait IntoHexStr<T> where T: LowerHex {
    fn into_hex_str(&self) -> String;
}


/*
impl IntoHexStr for usize {
    fn into_hex_str<usize>(&self) -> String {
        format!("{:#x}", self)
    }
}

*/

impl<T> IntoHexStr<T> for T where T: LowerHex {
    fn into_hex_str(&self) -> String {
        format!("{:#x}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_make_hex_strings() {
        let hex_str = "0xff"; // 255
        
        
        let mut test_str = 255_i32.into_hex_str();
        assert_eq!(hex_str, &test_str);

        test_str = 255_u32.into_hex_str(); 
        assert_eq!(hex_str, &test_str);
        
        test_str = 255_u64.into_hex_str();
        assert_eq!(hex_str, &test_str);
        
        test_str = 255_usize.into_hex_str();
        assert_eq!(hex_str, &test_str);
    }
}

