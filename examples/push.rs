use std::time::Duration;

use futures::SinkExt;
use tokio::time::delay_for;

use tmq::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut socket = push(&Context::new())
        .connect("tcp://127.0.0.1:3000")?
        .finish()?;

    let mut i = 0;
    loop {
        let message = format!("Push #{}", i);
        i += 1;

        println!("Push: {}", message);
        let multipart = Multipart::from(vec![zmq::Message::from(&message)]);
        socket.send(multipart).await?;
        delay_for(Duration::from_millis(1000)).await;
    }
}
