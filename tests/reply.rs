use zmq::{Context};

use futures::{SinkExt, StreamExt};
use tmq::{reply, Result};
use std::thread::{JoinHandle, spawn};
use utils::{
    generate_tcp_address, msg,
};

mod utils;

#[tokio::test]
async fn single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = reply(&ctx).bind(&address)?.finish()?;

    let part2 = "single_message";
    let echo = sync_requester(address, 1, part2);

    if let Some(multipart) = sock.next().await {
        let multipart = multipart?;
        assert_eq!(multipart.len(), 2);
        assert_eq!(multipart[1].as_str().unwrap(), part2);
        sock.send(multipart).await?;
    } else {
        panic!("Request is missing.");
    }

    echo.join().unwrap();

    Ok(())
}

// #[tokio::test] // disabled due to hang rather than error
async fn send_first_is_err() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = reply(&ctx).bind(&address)?.finish()?;

    let res = sock.send(vec![msg(b"Msg")]).await;
    assert!(res.is_err());

    Ok(())
}

// #[tokio::test] // disabled due to hang rather than error
async fn recv_2x_is_err() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = reply(&ctx).bind(&address)?.finish()?;

    let part2 = "single_message";
    let echo = sync_requester(address, 1, part2);

    sock.next().await.unwrap().unwrap();
    let res = sock.next().await.unwrap();
    assert!(res.is_err());

    echo.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn hammer_reply() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = reply(&ctx).bind(&address)?.finish()?;

    let count = 1_000;

    let part2 = "single_message";
    let echo = sync_requester(address, count, part2);

    for _ in 0..count {
        if let Some(multipart) = sock.next().await {
            let multipart = multipart?;
            assert_eq!(multipart.len(), 2);
            assert_eq!(multipart[1].as_str().unwrap(), part2);
            sock.send(multipart).await?;
        } else {
            panic!("Request is missing.");
        }
    }

    echo.join().unwrap();

    Ok(())
}

pub fn sync_requester(address: String, count: u64, part2: &'static str) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(zmq::SocketType::REQ).unwrap();
        socket.connect(&address).unwrap();

        for i in 0..count {
            let msg = format!("Req#{}",i);
            socket.send_multipart(&[msg.as_bytes(), part2.as_bytes()], 0).unwrap();
            let received = socket.recv_multipart(0).unwrap();
            assert_eq!(&received[0], &msg.as_bytes());
        }
    })
}