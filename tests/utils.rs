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

pub fn send_multiparts(address: String, socket_type: SocketType, multipart: Vec<Multipart>) -> JoinHandle<()>
{
    spawn(move || {
        let socket = Context::new().socket(socket_type).unwrap();
        socket.connect(&address).unwrap();

        for mp in multipart.into_iter() {
            socket.send_multipart(mp.into_iter(), 0).unwrap();
        }
    })
}

pub async fn assert_receive_all_multiparts<S: Stream<Item=Result<Multipart>> + Unpin>(
    mut stream: S,
    expected: Vec<Multipart>
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
    assert_eq!(data, expected);
    Ok(())
}
