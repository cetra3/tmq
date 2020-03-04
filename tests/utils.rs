#![allow(dead_code)]

use std::thread::{spawn, JoinHandle};

use futures::{Sink, SinkExt, Stream};
use zmq::{Context, SocketType};

use futures::StreamExt;
use rand::Rng;
use std::sync::{Arc, Barrier};
use tmq::{Multipart, Result, TmqError};

/// Synchronous send and receive functions running in a separate thread.
pub fn sync_send_multiparts<T: Into<zmq::Message> + Send + 'static>(
    address: String,
    socket_type: SocketType,
    multipart: Vec<Vec<T>>,
) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.connect(&address).unwrap();

        for mp in multipart.into_iter() {
            socket
                .send_multipart(mp.into_iter().map(|i| i.into()), 0)
                .unwrap();
        }
    })
}
pub fn sync_send_multipart_repeated<T: Into<zmq::Message> + Clone + 'static + Send>(
    address: String,
    socket_type: SocketType,
    multipart: Vec<T>,
    count: u64,
) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.connect(&address).unwrap();

        for _ in 0..count {
            let msg = multipart
                .clone()
                .into_iter()
                .map(|i| Into::<zmq::Message>::into(i));
            socket.send_multipart(msg, 0).unwrap();
        }
    })
}
pub fn sync_receive_multiparts<T: Into<zmq::Message> + Send + 'static>(
    address: String,
    socket_type: SocketType,
    expected: Vec<Vec<T>>,
) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.bind(&address).unwrap();

        for item in expected.into_iter() {
            let received: Multipart = socket
                .recv_multipart(0)
                .unwrap()
                .into_iter()
                .map(|i| i.into())
                .collect();
            assert_eq!(
                item.into_iter().map(|i| i.into()).collect::<Multipart>(),
                received
            );
        }
    })
}
pub fn sync_receive_multipart_repeated<T: Into<zmq::Message> + Send + 'static>(
    address: String,
    socket_type: SocketType,
    multipart: Vec<T>,
    count: u64,
) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.bind(&address).unwrap();

        let multipart: Multipart = multipart.into_iter().map(|i| i.into()).collect();
        for _ in 0..count {
            let received = socket.recv_multipart(0).unwrap();
            assert_eq!(
                multipart,
                received
                    .into_iter()
                    .map(|i| i.into())
                    .collect::<Multipart>()
            );
        }
    })
}
pub fn sync_receive_subscribe<T: Into<zmq::Message> + Send + 'static>(
    address: String,
    topic: String,
    expected: Vec<Vec<T>>,
) -> (JoinHandle<()>, Arc<Barrier>) {
    let barrier = Arc::new(Barrier::new(2));
    let handle = barrier.clone();
    (
        spawn(move || {
            let socket = Context::new().socket(zmq::SocketType::SUB).unwrap();
            socket.connect(&address).unwrap();
            socket.set_subscribe(topic.as_bytes()).unwrap();
            handle.wait();

            for item in expected.into_iter() {
                let received: Multipart = socket
                    .recv_multipart(0)
                    .unwrap()
                    .into_iter()
                    .map(|i| i.into())
                    .collect();
                assert_eq!(
                    item.into_iter().map(|i| i.into()).collect::<Multipart>(),
                    received
                );
            }
        }),
        barrier,
    )
}
pub fn sync_echo(address: String, socket_type: SocketType, count: u64) -> JoinHandle<()> {
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.bind(&address).unwrap();

        for _ in 0..count {
            let received = socket.recv_multipart(0).unwrap();
            socket.send_multipart(received, 0).unwrap();
        }
    })
}

/// Functions for sending and receiving using the asynchronous sockets.
pub async fn check_receive_multiparts<
    S: Stream<Item = Result<Multipart>> + Unpin,
    T: Into<zmq::Message>,
>(
    mut stream: S,
    expected: Vec<Vec<T>>,
) -> Result<()> {
    for item in expected.into_iter() {
        if let Some(msg) = stream.next().await {
            assert_eq!(
                msg?,
                item.into_iter().map(|i| i.into()).collect::<Multipart>()
            );
        } else {
            panic!("Stream ended too soon");
        }
    }
    Ok(())
}
pub async fn receive_multipart_repeated<
    S: Stream<Item = Result<Multipart>> + Unpin,
    T: Into<zmq::Message>,
>(
    mut stream: S,
    expected: Vec<T>,
    count: u64,
) -> Result<()> {
    let expected: Multipart = expected.into_iter().map(|i| i.into()).collect();
    for _ in 0..count {
        if let Some(msg) = stream.next().await {
            assert_eq!(msg?, expected);
        } else {
            panic!("Stream ended too soon");
        }
    }
    Ok(())
}
pub async fn send_multiparts<
    S: Sink<Multipart, Error = TmqError> + Unpin,
    T: Into<zmq::Message>,
>(
    mut sink: S,
    messages: Vec<Vec<T>>,
) -> Result<()> {
    for message in messages.into_iter() {
        sink.send(message.into_iter().map(|i| i.into()).collect::<Multipart>())
            .await?;
    }

    Ok(())
}
pub async fn send_multipart_repeated<
    S: Sink<Multipart, Error = TmqError> + Unpin,
    T: Into<zmq::Message> + Clone,
>(
    mut sink: S,
    message: Vec<T>,
    count: u64,
) -> Result<()> {
    for _ in 0..count {
        sink.send(
            message
                .clone()
                .into_iter()
                .map(|i| i.into())
                .collect::<Multipart>(),
        )
        .await?;
    }

    Ok(())
}
pub async fn hammer_receive<S: Stream<Item = Result<Multipart>> + Unpin>(
    stream: S,
    address: String,
    socket_type: SocketType,
) -> Result<()> {
    let count: u64 = 1_000_000;
    let thread = sync_send_multipart_repeated(address, socket_type, vec!["hello", "world"], count);

    receive_multipart_repeated(stream, vec!["hello", "world"], count).await?;

    thread.join().unwrap();

    Ok(())
}

/// Helper functions
pub fn generate_tcp_address() -> String {
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(2000, 65000);
    format!("tcp://127.0.0.1:{}", port)
}

pub fn msg(bytes: &[u8]) -> zmq::Message {
    zmq::Message::from(bytes)
}
