use futures::StreamExt;

use tmq::{subscribe, Context, Result};

use log::info;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "subscribe=DEBUG");
    }

    pretty_env_logger::init();

    let mut socket = subscribe(&Context::new())
        .connect("tcp://127.0.0.1:7899")?
        .subscribe(b"topic")?;

    while let Some(msg) = socket.next().await {
        info!(
            "Subscribe: {:?}",
            msg?.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
    Ok(())
}
