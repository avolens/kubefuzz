use std::env;

#[macro_export]
macro_rules! error_exit
{
    ($($arg:tt)*) => {{
        error!($($arg)*);
        std::process::exit(1);
    }}
}

pub fn initlog() {
    match env::var("RUST_LOG") {
        Ok(_) => {} // If the RUST_LOG variable is already set, do nothing.
        Err(_) => env::set_var("RUST_LOG", "info"), // If it's not set, set it to "error".
    }
    pretty_env_logger::init();
}
