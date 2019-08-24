#![feature(async_await)]

use zmq::{Context, SocketType};

use futures::SinkExt;
use std::thread::spawn;
use std::time::Duration;
use tmq::{push, Result, SocketExt};
use tokio::future::FutureExt;
use utils::{
    generate_tcp_addres, msg, send_multipart_repeated, send_multiparts,
    sync_receive_multipart_repeated, sync_receive_multiparts,
};

mod utils;

#[tokio::test]
async fn send_single_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::PULL,
        vec![vec![msg(b"hello"), msg(b"world")]],
    );

    send_multiparts(sock, vec![vec![msg(b"hello"), msg(b"world")]]).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_multiple_messages() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::PULL,
        vec![
            vec![msg(b"hello"), msg(b"world")],
            vec![msg(b"second"), msg(b"message")],
            vec![msg(b"third"), msg(b"message")],
        ],
    );

    send_multiparts(
        sock,
        vec![
            vec![msg(b"hello"), msg(b"world")],
            vec![msg(b"second"), msg(b"message")],
            vec![msg(b"third"), msg(b"message")],
        ],
    )
    .await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_empty_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::PULL,
        vec![vec![msg(b"hello"), msg(b"world")]],
    );

    send_multiparts(
        sock,
        vec![vec![], vec![], vec![], vec![msg(b"hello"), msg(b"world")]],
    )
    .await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_hammer() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = push(&ctx).connect(&address)?.finish();

    let count = 1_000;
    let thread = sync_receive_multipart_repeated(
        address,
        SocketType::PULL,
        vec![b"hello".to_vec(), b"world".to_vec()],
        count,
    );

    send_multipart_repeated(sock, vec![b"hello".to_vec(), b"world".to_vec()], count).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_delayed() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = push(&ctx).connect(&address)?.finish();

    // set send high water mark to a single message
    sock.set_sndhwm(1).unwrap();

    // send a single message, now the send buffer should be full
    sock.send(vec![msg(b"hello")]).await?;

    // assert that send will block now
    assert!(sock
        .send(vec!(msg(b"world")))
        .timeout(Duration::from_millis(200))
        .await
        .is_err());

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
