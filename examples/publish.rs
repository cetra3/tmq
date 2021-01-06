use tmq::{publish, Context, Result};

use futures::SinkExt;
use log::info;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "publish=DEBUG");
    }

    pretty_env_logger::init();

    let mut socket = publish(&Context::new()).bind("tcp://127.0.0.1:7899")?;

    let mut i = 0;
    loop {
        i += 1;
        let message = format!("Broadcast #{}", i);
        info!("Publish: {}", message);

        socket
            .send(vec![b"topic" as &[u8], message.as_bytes()])
            .await?;
        sleep(Duration::from_secs(1)).await;
    }
}
