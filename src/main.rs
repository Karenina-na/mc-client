use env_logger::{Builder, Target};
use lazy_static::lazy_static;

mod client;
mod itti;
mod util;

lazy_static! {
    static ref INIT: () = {
        std::env::set_var("RUST_LOG", "debug");
        let mut builder = Builder::from_default_env();
        builder.target(Target::Stdout);
        builder.is_test(true).init();
    };
}

#[tokio::main]
async fn main() {
    let _ = *INIT;
}
