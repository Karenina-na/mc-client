use env_logger::Target;
use log::{debug, error, info, log_enabled, warn, Level};

mod itti;
mod msg;
mod util;

#[tokio::main]
async fn main() {
    env_logger::builder().target(Target::Stdout).init();

    debug!("this is a debug {}", "message");
    info!("this is an info message");
    warn!("this is a warning");
    error!("this is printed by default");

    if log_enabled!(Level::Info) {
        let x = 3 * 4; // expensive computation
        info!("the answer was: {}", x);
    }
}
