#![feature(async_await)]

use futures::StreamExt;

use tmq::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut socket = pull(&Context::new()).bind("tcp://127.0.0.1:7899")?.finish();

    while let Some(msg) = socket.next().await {
        println!(
            "Pull: {:?}",
            msg?.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }

    Ok(())
}
