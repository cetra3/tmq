use log::info;
use std::env;
use tmq::{reply, Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "reply=DEBUG");
    }

    pretty_env_logger::init();

    let mut recv_sock = reply(&Context::new()).bind("tcp://127.0.0.1:7897")?;

    loop {
        let (multipart, send_sock) = recv_sock.recv().await?;
        info!(
            "Request: {:?}",
            multipart
                .iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
        recv_sock = send_sock.send(multipart).await?;
    }
}
