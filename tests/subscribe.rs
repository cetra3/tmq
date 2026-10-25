use tmq::{subscribe, BufferedReceiver, Result};
use zmq::{Context, SocketType};

use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;
use utils::generate_tcp_address;

mod utils;

#[tokio::test]
async fn receive_single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let topic: &[u8] = b"topic2";

    let mut sub_sock = subscribe(&ctx).connect(&address)?.subscribe(topic)?;
    let data = vec![topic, b"hello", b"world"];
    let pub_sock = Context::new().socket(SocketType::PUB).unwrap();
    pub_sock.bind(&address).unwrap();

    // Subscribe sockets don't know when they're connected by zmq design.
    // Sometimes there's a small delay, which can make this test unstable.
    // To work around it, try up to 5 times with a short timeout.

    for _ in 0usize..5 {
        pub_sock.send_multipart(&data, 0).unwrap();
        if let Ok(Some(Ok(incoming))) = timeout(Duration::from_millis(100), sub_sock.next()).await {
            assert_eq!(
                incoming,
                data.into_iter()
                    .map(tmq::Message::from)
                    .collect::<tmq::Multipart>()
            );
            return Ok(());
        }
    }

    panic!("Didn't receive published message");
}

#[tokio::test]
async fn receive_buffered_messages() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let topic: &[u8] = b"topic";

    let mut sub_sock: BufferedReceiver = subscribe(&ctx)
        .connect(&address)?
        .subscribe(topic)?
        .buffered(2);
    let pub_sock = Context::new().socket(SocketType::PUB).unwrap();
    pub_sock.bind(&address).unwrap();

    // Publish until the subscription is live, as in `receive_single_message`.
    let ready = vec![topic, b"ready"];
    let mut live = false;
    for _ in 0usize..5 {
        pub_sock.send_multipart(&ready, 0).unwrap();
        if let Ok(Some(Ok(_))) = timeout(Duration::from_millis(100), sub_sock.next()).await {
            live = true;
            break;
        }
    }
    assert!(live, "Didn't receive published message");

    // Send more messages than the buffer holds; they must arrive in order.
    let bodies: [&[u8]; 5] = [b"0", b"1", b"2", b"3", b"4"];
    for body in bodies {
        pub_sock.send_multipart([topic, body], 0).unwrap();
    }
    let mut received = Vec::new();
    while received.len() < bodies.len() {
        let msg = timeout(Duration::from_secs(1), sub_sock.next())
            .await
            .expect("Didn't receive buffered message")
            .expect("Stream ended too soon")?;
        if &msg[1][..] != b"ready" {
            received.push(msg[1].to_vec());
        }
    }
    assert_eq!(received, bodies.map(<[u8]>::to_vec));

    Ok(())
}
