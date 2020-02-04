use tmq::{subscribe, Result};
use zmq::{Context, SocketType};

use std::time::Duration;
use tokio::time::timeout;
use futures::StreamExt;
use utils::{generate_tcp_address};

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

    for _ in 0usize..5 {
        pub_sock.send_multipart(&data, 0).unwrap();
        if let Ok(Some(Ok(incoming))) = timeout(Duration::from_millis(100), sub_sock.next()).await {
            assert_eq!(incoming, data.into_iter().map(tmq::Message::from).collect::<tmq::Multipart>());
            return Ok(());
        }
    }

    panic!("Didn't receive published message");
}
