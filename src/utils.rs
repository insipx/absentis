use std::fmt::LowerHex;
use colored::Colorize;


crate trait IntoHexStr<T> where T: LowerHex {
    fn into_hex_str(&self) -> String;
}

impl<T> IntoHexStr<T> for T where T: LowerHex {
    fn into_hex_str(&self) -> String {
        format!("{:#x}", self)
    }
}

#[macro_export] 
macro_rules! err_loc {
    () => ({
        let occurred = "Occurred on".bright_red().bold().underline();
        let line_str = "line".bold().yellow().underline();
        let col_str = "col".bold().yellow().underline();
        let file_str = "in file".bold().yellow().underline();
        format!("{} {}: {}, {}: {}; {}: {}", occurred, line_str, line!().to_string().red().bold(), col_str, column!().to_string().red().bold(), file_str, file!().bright_white().bold()) 
    })
}

// Verbose error message
#[macro_export]
macro_rules! verb_err {
    ($err: ident) => ({
        
        match $err {
            Ok(_) => (),
            Err(e) => {
                error!("{}", e.to_string().bright_red().underline());
                error!("{}", err_loc!());
                return Err(e.into())
            }
        };
        $err.expect("If `$err` is an error value, we return immediately; qed")
    })
}

// a layer of 'format' for errors
#[macro_export]
macro_rules! new_err {
    ($str:expr, $($pms:expr),+ ) => ({
        format!($str,$( $pms ),+)
    })
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

