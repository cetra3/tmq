#![feature(async_await)]

use futures::{SinkExt, StreamExt};
use zmq::{Context, SocketType};

use tmq::{dealer, Multipart, Result};
use utils::{
    generate_tcp_addres, msg, send_multipart_repeated, send_multiparts, sync_echo,
    sync_receive_multipart_repeated, sync_receive_multiparts,
};

mod utils;

#[tokio::test]
async fn send_single_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::DEALER,
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
    let sock = dealer(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::DEALER,
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
    let sock = dealer(&ctx).connect(&address)?.finish();

    let thread = sync_receive_multiparts(
        address,
        SocketType::DEALER,
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
    let sock = dealer(&ctx).connect(&address)?.finish();

    let count = 1_000;
    let thread = sync_receive_multipart_repeated(
        address,
        SocketType::DEALER,
        vec![b"hello".to_vec(), b"world".to_vec()],
        count,
    );

    send_multipart_repeated(sock, vec![b"hello".to_vec(), b"world".to_vec()], count).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn proxy_sequence() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = dealer(&ctx).connect(&address)?.finish();

    let count = 1_000;
    let echo = sync_echo(address, SocketType::DEALER, count);

    let make_msg = |index| -> Multipart {
        let m1 = format!("Msg #{}", index);
        let m2 = format!("Msg #{} (contd.)", index);
        vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into()
    };

    for i in 0..count {
        sock.send(make_msg(i)).await?;
    }

    for i in 0..count {
        if let Some(multipart) = sock.next().await {
            let multipart = multipart?;

            assert_eq!(make_msg(i), multipart);
        } else {
            panic!("Stream ended too soon.");
        }
    }

    echo.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn proxy_interleaved() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = dealer(&ctx).connect(&address)?.finish();

    let count = 1_000;
    let echo = sync_echo(address, SocketType::DEALER, count);

    for i in 0..count {
        let m1 = format!("Msg #{}", i);
        let m2 = format!("Msg #{} (contd.)", i);
        sock.send(vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into())
            .await?;
        if let Some(multipart) = sock.next().await {
            let multipart = multipart?;

            let expected: Multipart = vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into();
            assert_eq!(expected, multipart);
        } else {
            panic!("Iteam in stream is missing.");
        }
    }

    echo.join().unwrap();

    Ok(())
}
