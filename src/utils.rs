
#[macro_export]
macro_rules! pretty_err {
    // colors entire string red underline and bold
    /*($format:expr, $($str:expr),+) => ({
        use colored::Colorize;
        format!($format, $($str.red().bold().underline()),+)
    }); */
    //colors first string bright red, bold, underline, rest dimmed
    ($format:expr, $str:expr, $($muted:expr),*) => ({
        use colored::Colorize;
        let string = format!($format, $str.bright_red().bold().underline(), $($muted.red().dimmed()),+);
        error!("{}", string);
    });

}
