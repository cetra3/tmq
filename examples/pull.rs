use futures::StreamExt;

use log::info;

use std::env;
use tmq::{pull, Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "subscribe=DEBUG");
    }

    pretty_env_logger::init();

    let mut socket = pull(&Context::new()).bind("tcp://127.0.0.1:7899")?;

    while let Some(msg) = socket.next().await {
        info!(
            "Pull: {:?}",
            msg?.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }

    Ok(())
}
