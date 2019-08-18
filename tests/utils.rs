#![feature(async_await)]

use std::thread::{JoinHandle, spawn};

use futures::Stream;
use zmq::{Context, SocketType};

use futures::StreamExt;
use tmq::{Multipart, Result};
use rand::Rng;

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

pub fn send_multiparts<T: Into<Multipart> + Send + 'static>(address: String,
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
pub fn send_multipart_repeated(address: String,
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
