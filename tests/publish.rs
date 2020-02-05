use std::time::Duration;
use tmq::{publish, Result};
use zmq::Context;

use utils::{generate_tcp_address, send_multiparts, sync_receive_subscribe};

mod utils;

#[tokio::test]
async fn send_single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = publish(&ctx).bind(&address)?;

    let topic = "topic2";
    let data = vec![vec![topic, "hello", "world"]];
    let (thread, barrier) = sync_receive_subscribe(address, topic.to_owned(), data.clone());

    barrier.wait();
    std::thread::sleep(Duration::from_millis(1000)); // hack to let the subscriber prepare
    send_multiparts(sock, data).await?;

    thread.join().unwrap();

    Ok(())
}
