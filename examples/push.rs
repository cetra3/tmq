use std::time::{Duration, Instant};

use futures::SinkExt;
use tokio::timer::Delay;

use tmq::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut socket = push(&Context::new())
        .connect("tcp://127.0.0.1:3000")?
        .finish();

    let mut i = 0;
    loop {
        let message = format!("Push #{}", i);
        i += 1;

        println!("Push: {}", message);
        let mut multipart = Multipart::new();
        multipart.push_back(zmq::Message::from(&message));
        socket.send(multipart).await?;
        Delay::new(Instant::now() + Duration::from_millis(1000)).await;
    }
}
