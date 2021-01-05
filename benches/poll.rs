#[macro_use]
extern crate criterion;

use criterion::Criterion;

use futures::{SinkExt, StreamExt};
use std::thread::spawn;
use tmq::{pull, push, SocketExt};
use zmq::SocketType;

fn poll_benchmark(c: &mut Criterion) {
    c.bench_function("receive", |b| {
        let address = "tcp://127.0.0.1:3011";
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let ctx = zmq::Context::new();
        let sender = ctx.socket(SocketType::PUSH).unwrap();
        sender.set_linger(0).unwrap();
        sender.connect(address).unwrap();

        let ctx2 = zmq::Context::new();
        let mut socket = {
            let _guard = runtime.enter();
            pull(&ctx2).bind(address).unwrap()
        };

        b.iter_with_setup(
            || {
                sender
                    .send_multipart(
                        vec![zmq::Message::from("hello"), zmq::Message::from("world")].into_iter(),
                        0,
                    )
                    .unwrap();
            },
            |_| {
                runtime.block_on(socket.next()).unwrap().unwrap();
            },
        );
    });

    c.bench_function("send", |b| {
        let address = "tcp://127.0.0.1:3011";
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let thread = spawn(move || {
            let ctx = zmq::Context::new();
            let receiver = ctx.socket(SocketType::PULL).unwrap();
            receiver.bind(address).unwrap();

            let mut counter = 0;
            let mut target = -1;
            loop {
                let msg = receiver.recv_multipart(0).unwrap();
                if msg.len() == 1 {
                    target = std::str::from_utf8(&msg[0]).unwrap().parse().unwrap();
                } else {
                    counter += 1;
                }
                if counter == target {
                    break;
                }
            }
        });

        let ctx = zmq::Context::new();
        let mut socket = {
            let _guard = runtime.enter();
            push(&ctx).connect(address).unwrap()
        };
        socket.set_linger(0).unwrap();

        let mut sent = 0;
        b.iter_with_setup(
            || vec![zmq::Message::from("hello"), zmq::Message::from("world")],
            |msg| {
                runtime.block_on(socket.send(msg)).unwrap();
                sent += 1;
            },
        );

        runtime
            .block_on(socket.send(vec![zmq::Message::from(&sent.to_string())]))
            .unwrap();
        thread.join().unwrap();
    });
}

criterion_group!(benches, poll_benchmark);
criterion_main!(benches);
