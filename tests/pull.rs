use zmq::{Context, SocketType};

use futures::StreamExt;
use std::{
    sync::{Arc, Barrier},
    thread::spawn,
};
use tmq::{pull, Result};
use utils::{check_receive_multiparts, generate_tcp_address, hammer_receive, sync_send_multiparts};

mod utils;

#[tokio::test]
async fn receive_single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?;

    let thread = sync_send_multiparts(address, SocketType::PUSH, vec![vec!["hello", "world"]]);

    check_receive_multiparts(sock, vec![vec!["hello", "world"]]).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_multiple_messages() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?;

    let thread = sync_send_multiparts(
        address,
        SocketType::PUSH,
        vec![vec!["hello", "world"], vec!["second", "message"]],
    );

    check_receive_multiparts(
        sock,
        vec![vec!["hello", "world"], vec!["second", "message"]],
    )
    .await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?;
    hammer_receive(sock, address, SocketType::PUSH).await
}

#[tokio::test]
async fn receive_buffered_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?;
    hammer_receive(sock.buffered(1024), address, SocketType::PUSH).await
}

#[tokio::test]
async fn receive_delayed() -> Result<()> {
    let address = generate_tcp_address();
    let address_recv = address.clone();
    let barrier = Arc::new(Barrier::new(2));
    let barrier_send = barrier.clone();

    let thread = spawn(move || {
        let ctx = Context::new();
        let socket = ctx.socket(SocketType::PUSH).unwrap();
        socket.connect(&address).unwrap();
        for _ in 0..3 {
            socket.send_multipart(vec!["hello", "world"], 0).unwrap();
        }

        barrier_send.wait();
    });

    barrier.wait();

    let ctx = Context::new();
    let mut sock = pull(&ctx).set_rcvhwm(1).bind(&address_recv)?;

    for _ in 0..3 {
        sock.next().await.unwrap()?;
    }

    thread.join().unwrap();

    Ok(())
}
