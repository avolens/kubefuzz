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
        Ok(_) => {
            let curlevel = env::var("RUST_LOG").unwrap();
            env::set_var(
                "RUST_LOG",
                format!("{},kube_client=off,tower=off,hyper=off", curlevel),
            );
        }
        Err(_) => env::set_var("RUST_LOG", "info,kube_client=off,tower=off,hyper=off"),
    }
    pretty_env_logger::init();
}
