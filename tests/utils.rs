#![feature(async_await)]

use std::thread::{JoinHandle, spawn};

use futures::{Stream, Sink};
use zmq::{Context, SocketType};

use futures::StreamExt;
use tmq::{Multipart, Result, TmqError};
use rand::Rng;

/// Synchronous send and receive functions running in a separate thread.
pub fn sync_send_multiparts<T: Into<Multipart> + Send + 'static>(address: String,
                                                  socket_type: SocketType,
                                                  multipart: Vec<T>) -> JoinHandle<()>
{
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.connect(&address).unwrap();

        for mp in multipart.into_iter() {
            socket.send_multipart(mp.into().into_iter(), 0).unwrap();
        }
    })
}
pub fn sync_send_multipart_repeated(address: String,
                               socket_type: SocketType,
                               multipart: Vec<Vec<u8>>,
                               count: u64) -> JoinHandle<()>
{
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.connect(&address).unwrap();

        for _ in 0..count {
            let msg = multipart.clone().into_iter().map(|i| i.into()).collect::<Vec<zmq::Message>>();
            socket.send_multipart(msg, 0).unwrap();
        }
    })
}
pub fn sync_receive_multiparts<T: Into<Multipart>>(address: String,
                               socket_type: SocketType,
                               expected: Vec<T>) -> JoinHandle<()>
{
    let expected = expected.into_iter().map(|i| i.into()).collect::<Vec<Multipart>>();
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
            assert_eq!(item, received);
        }
    })
}
pub fn sync_receive_multipart_repeated(address: String,
                                       socket_type: SocketType,
                                       multipart: Vec<Vec<u8>>,
                                       count: u64) -> JoinHandle<()>
{
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.bind(&address).unwrap();

        for _ in 0..count {
            let received = socket.recv_multipart(0).unwrap();
            assert_eq!(multipart, received);
        }
    })
}

/// Functions for sending and receiving using the asynchronous sockets.
pub async fn check_receive_multiparts<
    S: Stream<Item=Result<Multipart>> + Unpin,
    T: Into<Multipart>>
(
    mut stream: S,
    expected: Vec<T>
) -> Result<()>
{
    let mut data = Vec::new();

    for _ in 0..expected.len() {
        if let Some(msg) = stream.next().await
        {
            data.push(msg?)
        }
        else
        {
            panic!("Stream ended too early");
        }
    }
    assert_eq!(data, expected.into_iter().map(|i| i.into()).collect::<Vec<Multipart>>());
    Ok(())
}
pub async fn receive_multipart_repeated<
    S: Stream<Item=Result<Multipart>> + Unpin,
    T: Into<Multipart>>
(
    mut stream: S,
    expected: T,
    count: u64
) -> Result<()>
{
    let expected = expected.into();
    for _ in 0..count {
        if let Some(msg) = stream.next().await
        {
            assert_eq!(msg?, expected);
        }
        else
        {
            panic!("Stream ended too early");
        }
    }
    Ok(())
}
pub async fn send_multiparts<S: Sink<Multipart, Error=TmqError>, T: Into<Multipart>>(sink: S,
                                                                     messages: Vec<T>) -> Result<()>
{
    futures::stream::iter(messages.into_iter().map(|i| Ok(i.into())))
        .forward(sink)
        .await?;
    Ok(())
}
pub async fn send_multipart_repeated<S: Sink<Multipart, Error=TmqError>>(
    sink: S,
    message: Vec<Vec<u8>>,
    count: u64
) -> Result<()>
{
    let iterator = std::iter::repeat_with(|| Ok(message.clone().into_iter().map(|i| i.into()).collect::<Multipart>()))
        .take(count as usize);
    futures::stream::iter(iterator).forward(sink).await?;
    Ok(())
}

/// Helper functions
pub fn generate_tcp_addres() -> String
{
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(2000, 65000);
    format!("tcp://127.0.0.1:{}", port)
}

pub fn msg(bytes: &[u8]) -> zmq::Message
{
    zmq::Message::from(bytes)
}
