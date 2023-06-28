//! This example demonstrates asynchronous distribution of requests given by a set of clients
//! among a set of workers. The clients communicate with a ROUTER broker, which load balances their
//! requests among the workers.
//!
//! Each client continuously creates a request and sends it to the proxy.
//! The proxy forwards it to a worker. Since the proxy is a ROUTER socket, it will use the identity
//! of the client socket as the first message in the multipart sent to the worker. The worker will
//! simulate some amount of work and respond with the client identity followed by some payload. The
//! proxy will then use the identity to respond to the correct client.
//!
//! All clients, workers and the proxy run on the same thread.
//!
//! The proxy is implemented using asynchronous sockets as an example of using `futures::select`.
//! A more performant solution would be to use `zmq::proxy`, which is designed for this usage.

use futures::{future, SinkExt, StreamExt};
use rand::Rng;
use std::{error::Error, rc::Rc, time::Duration};
use tmq::{dealer, router, Context, Multipart};
use tokio::time::sleep;

async fn client(ctx: Rc<Context>, client_id: u64, frontend: String) -> tmq::Result<()> {
    let mut sock = dealer(&ctx).connect(&frontend)?;
    let mut rng = rand::thread_rng();

    let client_id = client_id.to_string();
    let mut request_id = 0;
    loop {
        println!("Client {} sending request {}", client_id, request_id);

        let request_id_str = request_id.to_string();
        let msg = vec![client_id.as_bytes(), request_id_str.as_bytes(), b"request"];
        sock.send(msg).await?;

        let response = sock.next().await.unwrap()?;
        let expected: Multipart =
            vec![client_id.as_bytes(), request_id_str.as_bytes(), b"response"].into();
        assert_eq!(expected, response);

        let sleep_time = rng.gen_range(200..1000);
        sleep(Duration::from_millis(sleep_time)).await;
        request_id += 1;
    }
}
async fn worker(ctx: Rc<Context>, worker_id: u64, backend: String) -> Result<(), Box<dyn Error>> {
    let mut sock = dealer(&ctx).connect(&backend)?;
    let mut rng = rand::thread_rng();

    loop {
        let mut request = sock.next().await.unwrap()?;
        let identity = request.pop_front().unwrap();
        let client_id = request.pop_front().unwrap();
        let request_id = request.pop_front().unwrap();
        let request_body = request.pop_front().unwrap();

        println!(
            "Worker {} handling request(id={} body={}) from client {}",
            worker_id,
            request_id.as_str().unwrap(),
            request_body.as_str().unwrap(),
            client_id.as_str().unwrap()
        );

        // simulate work
        let sleep_time = rng.gen_range(100..3000);
        sleep(Duration::from_millis(sleep_time)).await;

        let response = vec![identity, client_id, request_id, "response".into()];
        sock.send(response).await?;
    }
}

/// Simulates zmq::proxy using asynchronous sockets.
async fn proxy(ctx: Rc<Context>, frontend: String, backend: String) -> tmq::Result<()> {
    let (mut router_tx, mut router_rx) = router(&ctx).bind(&frontend)?.split();
    let (mut dealer_tx, mut dealer_rx) = dealer(&ctx).bind(&backend)?.split();

    let mut frontend_fut = router_rx.next();
    let mut backend_fut = dealer_rx.next();

    loop {
        let msg = future::select(frontend_fut, backend_fut).await;
        match msg {
            future::Either::Left(router_msg) => {
                // proxy received a message from a client
                dealer_tx.send(router_msg.0.unwrap()?).await?;
                frontend_fut = router_rx.next();
                backend_fut = router_msg.1;
            }
            future::Either::Right(dealer_msg) => {
                // proxy received a message from a worker
                router_tx.send(dealer_msg.0.unwrap()?).await?;
                backend_fut = dealer_rx.next();
                frontend_fut = dealer_msg.1;
            }
        }
    }
}

fn main() -> tmq::Result<()> {
    let frontend = "tcp://127.0.0.1:5555".to_string();
    let backend = "tcp://127.0.0.1:5556".to_string();
    let ctx = Rc::new(Context::new());

    let mut runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let tasks = tokio::task::LocalSet::new();

    // spawn workers
    for worker_id in 0..2 {
        let ctx = ctx.clone();
        let backend = backend.clone();
        tasks.spawn_local(async move {
            worker(ctx, worker_id, backend)
                .await
                .expect("Worker failed");
        });
    }

    // spawn clients
    for client_id in 10..13 {
        let ctx = ctx.clone();
        let frontend = frontend.clone();
        tasks.spawn_local(async move {
            client(ctx, client_id, frontend)
                .await
                .expect("Client failed");
        });
    }

    tasks.block_on(&mut runtime, async move {
        proxy(ctx.clone(), frontend, backend)
            .await
            .expect("Proxy failed");
    });

    Ok(())
}
