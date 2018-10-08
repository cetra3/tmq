extern crate futures;
extern crate pretty_env_logger;
extern crate tmq;
extern crate tokio;

#[macro_use]
extern crate log;

use futures::{stream, Future, Stream};


use tmq::*;

use std::env;

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "request=DEBUG");
    }

    pretty_env_logger::init();

    let request = make_request(5);


    let request = req(&Context::new())
        .connect("tcp://127.0.0.1:7899")
        .expect("Couldn't connect")
        .with(request)
        .for_each(|val| {
            info!("Response: {}", val.as_str().unwrap_or(""));
            Ok(())
        }).map_err(|err| {
            error!("Error with request: {}", err);
        });

    tokio::run(request);

}

fn make_request(count: usize) -> impl Stream<Item=Message, Error=()> {

    let mut vec = Vec::new();

    for i in 0..count {
        vec.push(Message::from(&format!("Request #{}", i)));
    }

    stream::iter_ok(vec)
}
