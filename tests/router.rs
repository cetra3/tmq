use futures::{SinkExt, StreamExt};
use zmq::{Context, SocketType};

use tmq::{dealer, router, Multipart, Result};

use futures::Stream;
use std::thread::{spawn, JoinHandle};
use utils::{generate_tcp_address, sync_send_multipart_repeated, sync_send_multiparts};

mod utils;

#[tokio::test]
async fn receive_single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = router(&ctx).bind(&address)?;

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
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = router(&ctx).bind(&address)?;

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

async fn router_receive_hammer<S: Stream<Item = Result<Multipart>> + Unpin>(
    mut stream: S,
    address: String,
) -> Result<()> {
    let count: u64 = 1_000_000;
    let data = vec!["hello", "world"];
    let thread = sync_send_multipart_repeated(address, SocketType::DEALER, data.clone(), count);

    for _ in 0..count {
        let mut message = stream.next().await.unwrap()?;
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

#[tokio::test]
async fn receive_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = router(&ctx).bind(&address)?;
    router_receive_hammer(sock, address).await
}

#[tokio::test]
async fn receive_buffered_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let sock = router(&ctx).bind(&address)?;
    let (rx, _) = sock.split();
    router_receive_hammer(rx.buffered(1024), address).await
}

#[tokio::test]
async fn proxy() -> Result<()> {
    let frontend = generate_tcp_address();
    let backend = generate_tcp_address();
    let ctx = Context::new();
    let router = router(&ctx).bind(&frontend)?;
    let dealer = dealer(&ctx).bind(&backend)?;

    let count: u64 = 10_000;
    let client_count: u64 = 3;
    let worker_count: u64 = 2;
    let task_count: u64 = client_count * count;

    let clients = (0..client_count)
        .map(|client_id| {
            let address = frontend.clone();
            spawn(move || {
                let ctx = Context::new();
                let sock = ctx.socket(SocketType::DEALER).unwrap();
                sock.connect(&address).unwrap();

                let client_id = client_id.to_string();
                for index in 0..count {
                    let msg_index = index.to_string();
                    let msg = vec!["hello", "from", "client", &client_id, &msg_index];
                    sock.send_multipart(msg.clone().into_iter(), 0).unwrap();
                    let response = sock.recv_multipart(0).unwrap();
                    assert_eq!(
                        msg,
                        response
                            .iter()
                            .map(|i| std::str::from_utf8(&*i).unwrap())
                            .collect::<Vec<&str>>()
                    );
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    let workers = (0..worker_count)
        .map(|_| {
            let address = backend.clone();
            spawn(move || {
                let ctx = Context::new();
                let sock = ctx.socket(SocketType::DEALER).unwrap();
                sock.connect(&address).unwrap();

                loop {
                    let response = sock.recv_multipart(0).unwrap();
                    if response.len() == 1 {
                        break;
                    }
                    sock.send_multipart(response, 0).unwrap();
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    // simulates zmq::proxy
    let (mut router_rx, mut router_tx) = router.split();
    let (mut dealer_rx, mut dealer_tx) = dealer.split();
    let mut frontend_fut = router_rx.next();
    let mut backend_fut = dealer_rx.next();
    for _ in 0..(task_count * 2) {
        let msg = futures::future::select(frontend_fut, backend_fut).await;
        match msg {
            futures::future::Either::Left(router_msg) => {
                dealer_tx.send(router_msg.0.unwrap()?).await?;
                frontend_fut = router_rx.next();
                backend_fut = router_msg.1;
            }
            futures::future::Either::Right(dealer_msg) => {
                router_tx.send(dealer_msg.0.unwrap()?).await?;
                backend_fut = dealer_rx.next();
                frontend_fut = dealer_msg.1;
            }
        }
    }

    for client in clients {
        client.join().unwrap();
    }
    for _ in 0..worker_count {
        dealer_tx.send(vec!["end"]).await?;
    }
    for worker in workers {
        worker.join().unwrap();
    }

    Ok(())
}
