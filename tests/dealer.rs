use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use zmq::{Context, SocketType};

use std::thread::spawn;
use tmq::{dealer, Multipart, Result};
use tokio::prelude::Stream;
use utils::{
    generate_tcp_addres, hammer_receive, msg, receive_multipart_repeated, send_multipart_repeated,
    send_multiparts, sync_echo, sync_receive_multipart_repeated, sync_receive_multiparts,
    sync_send_multipart_repeated,
};

mod utils;

#[tokio::test]
async fn send_single_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).connect(&address)?.finish();

    let data = vec![vec!["hello", "world"]];
    let thread = sync_receive_multiparts(address, SocketType::DEALER, data.clone());

    send_multiparts(sock, data).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_multiple_messages() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).connect(&address)?.finish();

    let data = vec![
        vec!["hello", "world"],
        vec!["second", "message"],
        vec!["third", "message"],
    ];
    let thread = sync_receive_multiparts(address, SocketType::DEALER, data.clone());

    send_multiparts(sock, data).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_empty_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).connect(&address)?.finish();

    let data = vec!["hello", "world"];
    let thread = sync_receive_multiparts(address, SocketType::DEALER, vec![data.clone()]);

    send_multiparts(sock, vec![vec![], vec![], vec![], data]).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn send_hammer() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).connect(&address)?.finish();

    let count = 1_000;
    let data = vec!["hello", "world"];
    let thread = sync_receive_multipart_repeated(address, SocketType::DEALER, data.clone(), count);

    send_multipart_repeated(sock, data, count).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_hammer() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).bind(&address)?.finish();
    hammer_receive(sock, address, SocketType::DEALER).await
}

#[tokio::test]
async fn receive_buffered_hammer() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = dealer(&ctx).bind(&address)?.finish();
    let (rx, tx) = sock.split();
    hammer_receive(rx.buffered(1024), address, SocketType::DEALER).await
}

#[tokio::test]
async fn proxy_sequence() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = dealer(&ctx).connect(&address)?.finish();

    let count = 1_000;

    let make_msg = |index| -> Multipart {
        let m1 = format!("Msg #{}", index);
        let m2 = format!("Msg #{} (contd.)", index);
        vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into()
    };

    for i in 0..count {
        sock.send(make_msg(i)).await?;
    }

    let echo = sync_echo(address, SocketType::DEALER, count);

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
        sock.send(vec![msg(m1.as_bytes()), msg(m2.as_bytes())])
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

#[test]
fn split_echo() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let (rx, tx) = dealer(&ctx).bind(&address)?.finish().split();
    let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();

    let count = 10;
    runtime.spawn(
        rx.take(count)
            .forward(tx)
            .map_err(|e| panic!(e))
            .map(|_| ()),
    );

    let thread = spawn(move || {
        let ctx = Context::new();
        let sender = ctx.socket(SocketType::DEALER).unwrap();
        sender.connect(&address).unwrap();

        for _ in 0..count {
            sender
                .send_multipart(vec!["hello", "world"].into_iter(), 0)
                .unwrap();
            assert_eq!(
                sender.recv_multipart(0).unwrap(),
                vec!(b"hello".to_vec(), b"world".to_vec())
            );
        }
    });

    runtime.run().unwrap();
    thread.join().unwrap();

    Ok(())
}

#[test]
fn split_send_all() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let (rx, mut tx) = dealer(&ctx).connect(&address)?.finish().split();
    drop(rx);

    let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();

    let count = 10_000;
    let thread = spawn(move || {
        let ctx = Context::new();
        let receiver = ctx.socket(SocketType::DEALER).unwrap();
        receiver.bind(&address).unwrap();

        for i in 0..count {
            let received = receiver.recv_multipart(0).unwrap();
            assert_eq!(
                received,
                vec!(
                    i.to_string().as_bytes().to_vec(),
                    (i + 1).to_string().as_bytes().to_vec(),
                )
            );
        }
    });

    let mut count = futures::stream::iter((0..count).map(|i| {
        Multipart::from(vec![
            zmq::Message::from(&i.to_string()),
            zmq::Message::from(&(i + 1).to_string()),
        ])
    }));
    runtime.block_on(tx.send_all(&mut count))?;

    thread.join().unwrap();

    Ok(())
}
