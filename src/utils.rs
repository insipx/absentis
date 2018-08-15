use web3::BatchTransport;
use futures::future::Future;
use super::client::Client;

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! green {
    ($($strs: expr),+) => ({
        use colored::Colorize;
        $($strs.bright_green()),+
    });
}
#[macro_export]
macro_rules! format_num {
    ($num:expr) => ({
        use separator::Separatable;
        $num.separated_string()
    });
}

#[macro_export]
macro_rules! pretty_err {
    // colors entire string red underline and bold
    /*($format:expr, $($str:expr),+) => ({
        use colored::Colorize;
        format!($format, $($str.red().bold().underline()),+)
    }); */
    //colors first string bright red, bold, underline, rest dimmed
    ($format:expr, $str:expr, $($muted:expr),*) => ({
        use log::{error, log};
        use colored::Colorize;
        let string = format!($format, $str.bright_red().bold().underline(), $($muted.red().dimmed()),+);
        error!("{}", string);
    });

}

#[macro_export]
macro_rules! pretty_info {
    ($frmt:expr, $($strs:expr),+) => ({
        let string = format!($frmt, $(green!($strs)),+);
        info!("{}", string);
    });
}


#[macro_export]
macro_rules! infura_url {
    ($api_key:expr) => ({
        use super::types::INFURA_URL;
        format!("{}{}", INFURA_URL, $api_key)
    });
}

pub fn latest_block<T>(client: &Client<T>) -> u64 
    where
        T: BatchTransport
{
    let b = client.web3.eth().block_number().wait();
    let b = match b {
        Ok(v) => v,
        Err(e) => {
            pretty_err!("{}{}", "Could not get latest block: ", e.description());
            if let Some(bt) = e.backtrace() {
                pretty_err!("{}: {:?}", "Backtrace", format!("{:?}", bt));
            }
            panic!("Shutting down...");
        }

    };
    b.as_u64()
}


