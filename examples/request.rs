use futures::{SinkExt, StreamExt};
use log::info;
use std::env;
use tmq::{request, Context, Multipart, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "request=DEBUG");
    }

    pretty_env_logger::init();

    let mut socket = request(&Context::new())
        .connect("tcp://127.0.0.1:7897")?
        .finish()?;

    let mut i = 0u32;
    loop {
        let message = format!("Req#{}", i);
        i += 1;

        info!("Request: {:?}", &message);
        let multipart = Multipart::from(vec![zmq::Message::from(message.as_bytes())]);
        socket.send(multipart).await?;
        let msg = socket.next().await.unwrap()?;
        info!(
            "Reply: {:?}",
            msg.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}
