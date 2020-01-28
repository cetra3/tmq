use log::info;
use std::env;
use tmq::{request, Context, Multipart, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "request=DEBUG");
    }

    pretty_env_logger::init();

    let mut send_sock = request(&Context::new())
        .connect("tcp://127.0.0.1:7897")?
        .finish()?;

    let mut i = 0u32;
    loop {
        let message = format!("Req#{}", i);
        i += 1;

        info!("Request: {:?}", &message);
        let mut multipart = Multipart::from(vec![zmq::Message::from(message.as_bytes())]);
        let recv_sock = send_sock.send(&mut multipart).await?;
        let (msg,send) = recv_sock.recv().await?;
        send_sock = send;
        info!(
            "Reply: {:?}",
            msg.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}