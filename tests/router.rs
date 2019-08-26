#![feature(async_await)]

use futures::{SinkExt, StreamExt};
use zmq::{Context, SocketType};

use std::thread::spawn;
use std::time::Duration;
use tmq::{router, Multipart, Result, SocketExt};
use tokio::future::FutureExt;
use utils::{
    check_receive_multiparts, generate_tcp_addres, msg, send_multipart_repeated, send_multiparts,
    sync_echo, sync_receive_multipart_repeated, sync_receive_multiparts,
    sync_send_multipart_repeated, sync_send_multiparts,
};

mod utils;

#[tokio::test]
async fn receive_single_message() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = router(&ctx).bind(&address)?.finish();

    let data = vec!["hello", "world"];
    let thread = sync_send_multiparts(address, SocketType::DEALER, vec![data.clone()]);

    let mut message = sock.next().await.unwrap()?;
    assert_eq!(message.len(), 3);
    message.pop_front().unwrap();
    assert_eq!(
        message,
        data.into_iter().map(|i| i.into()).collect::<Multipart>()
    );

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_multiple_messages() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = router(&ctx).bind(&address)?.finish();

    let data = vec![vec!["hello", "world"], vec!["second", "message"]];

    let thread = sync_send_multiparts(address, SocketType::DEALER, data.clone());

    for item in data.into_iter() {
        let mut message = sock.next().await.unwrap()?;
        assert_eq!(message.len(), 3);
        message.pop_front().unwrap();
        assert_eq!(
            message,
            item.into_iter().map(|i| i.into()).collect::<Multipart>()
        );
    }

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_hammer() -> Result<()> {
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let mut sock = router(&ctx).bind(&address)?.finish();

    let count: u64 = 1_000_000;
    let data = vec!["hello", "world"];
    let thread = sync_send_multipart_repeated(address, SocketType::DEALER, data.clone(), count);

    for _ in 0..count {
        let mut message = sock.next().await.unwrap()?;
        assert_eq!(message.len(), 3);
        message.pop_front().unwrap();
        assert_eq!(
            message,
            data.iter().map(|i| i.into()).collect::<Multipart>()
        );
    }

    thread.join().unwrap();

    Ok(())
}
