use zmq::{Context, SocketType};

use futures::SinkExt;
use std::{thread::spawn, time::Duration};
use tmq::{push, Result};
use tokio::time::timeout;
use utils::{
    generate_tcp_address, msg, send_multipart_repeated, send_multiparts,
    sync_receive_multipart_repeated, sync_receive_multiparts,
};

mod utils;

#[tokio::test]
async fn send_single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?;

    let thread = sync_receive_multiparts(address, SocketType::PULL, vec![vec!["hello", "world"]]);

    send_multiparts(sock, vec![vec!["hello", "world"]]).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_multiple_messages() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?;

    let data = vec![
        vec!["hello", "world"],
        vec!["second", "message"],
        vec!["third", "message"],
    ];

    let thread = sync_receive_multiparts(address, SocketType::PULL, data.clone());

    send_multiparts(sock, data).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_empty_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?;

    let data = vec!["hello", "world"];
    let thread = sync_receive_multiparts(address, SocketType::PULL, vec![data.clone()]);

    send_multiparts(sock, vec![vec![], vec![], vec![], data]).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?;

    let count = 1_000;
    let data = vec!["hello", "world"];
    let thread = sync_receive_multipart_repeated(address, SocketType::PULL, data.clone(), count);

    send_multipart_repeated(sock, data, count).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_delayed() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();

    // set send high water mark to a single message
    let mut sock = push(&ctx).set_sndhwm(1).connect(&address)?;

    // send a single message, now the send buffer should be full
    sock.send(vec![msg(b"hello")]).await?;

    // assert that send will block now
    assert!(
        timeout(Duration::from_millis(200), sock.send(vec!(msg(b"world"))))
            .await
            .is_err()
    );

    // start the receiver
    let thread = spawn(move || {
        let ctx = Context::new();
        let socket = ctx.socket(SocketType::PULL).unwrap();
        socket.bind(&address).unwrap();

        for _ in 0..3 {
            socket.recv_multipart(0).unwrap();
        }
    });

    sock.send(vec![msg(b"world")]).await?;
    thread.join().unwrap();

    Ok(())
}
