use zmq::Context;
use tmq::{reply, Result};
use std::thread::{JoinHandle, spawn};
use utils::generate_tcp_address;

mod utils;

#[tokio::test]
async fn single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let recv_sock = reply(&ctx).bind(&address)?.finish()?;

    let part2 = "single_message";
    let echo = sync_requester(address, 1, part2);

    let (mut multipart,send_sock) = recv_sock.recv().await?;
    assert_eq!(multipart.len(), 2);
    assert_eq!(multipart[1].as_str().unwrap(), part2);
    send_sock.send(&mut multipart).await?;

    echo.join().unwrap();

    Ok(())
}


#[tokio::test]
async fn hammer_reply() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut recv_sock = reply(&ctx).bind(&address)?.finish()?;

    let count = 1_000;

    let part2 = "single_message";
    let echo = sync_requester(address, count, part2);

    for _ in 0..count {
        let (mut multipart,send_sock) = recv_sock.recv().await?;
        assert_eq!(multipart.len(), 2);
        assert_eq!(multipart[1].as_str().unwrap(), part2);
        recv_sock = send_sock.send(&mut multipart).await?;
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