use std::time::Duration;

use futures::SinkExt;
use tokio::time::sleep;

use log::info;

use std::env;
use tmq::{push, Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "subscribe=DEBUG");
    }

    pretty_env_logger::init();

    let mut socket = push(&Context::new()).connect("tcp://127.0.0.1:3000")?;

    let mut i = 0;
    loop {
        let message = format!("Push #{}", i);
        i += 1;

        info!("Push: {}", message);
        let multipart = vec![message.as_bytes()];
        socket.send(multipart).await?;
        sleep(Duration::from_millis(1000)).await;
    }
}
