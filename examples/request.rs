use log::info;
use std::env;
use tmq::{request, Context, Message, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "request=DEBUG");
    }

    pretty_env_logger::init();

    let mut send_sock = request(&Context::new()).connect("tcp://127.0.0.1:7897")?;

    let mut i = 0u32;
    loop {
        let message = format!("Req#{}", i);
        i += 1;

        info!("Request: {:?}", &message);
        let message: Message = message.as_bytes().into();
        let recv_sock = send_sock.send(message.into()).await?;
        let (msg, send) = recv_sock.recv().await?;
        send_sock = send;
        info!(
            "Reply: {:?}",
            msg.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}
