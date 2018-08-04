use std::fmt::LowerHex;

crate trait IntoHexStr<T> where T: LowerHex {
    fn into_hex_str(&self) -> String;
}

impl<T> IntoHexStr<T> for T where T: LowerHex {
    fn into_hex_str(&self) -> String {
        format!("{:#x}", self)
    }
}

#[macro_export]
macro_rules! pretty_err {
    ($err:ident) => ({
        use colored::Colorize;
        let err_string = format!("{}", $err);
        error!("{}", err_string.bright_red().bold().underline());
    })
}

#[macro_export]
macro_rules! pretty_success {
    ($succ:ident) => ({
        use colored::Colorize;
        let succ_string = format!("{}", $succ);
        info!("{}", succ_string.bright_green().bold().underline());
    })
}


#[macro_export] 
macro_rules! err_loc {
    () => ({
        use colored::Colorize;
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
    use log::{log, error};
    use colored::Colorize;
        
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
// TODO: Remove this #p2
#[macro_export]
macro_rules! new_err {
    ($str:expr, $($pms:expr),+ ) => ({
        format!($str,$( $pms ),+)
    })
}

// expects a string and value 
#[macro_export]
macro_rules! mismatched_types {
    ($expected_type: expr, $recvd_type: ident) => ({
        let string = format!("Expected type `{}`, got `{}` in {}", $expected_type, $recvd_type, err_loc!());
        Err(ResponseBuildError::MismatchedTypes(TypeMismatchError::new(string)))
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

