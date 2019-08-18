#![feature(async_await)]

use zmq::{Context, SocketType};

use tmq::{push, Result};
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
