extern crate futures;
extern crate pretty_env_logger;
extern crate tmq;
extern crate tokio;

#[macro_use]
extern crate log;

extern crate failure;

use futures::{Future, Stream};

use tmq::*;

use std::env;

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "subscribe=DEBUG");
    }

    pretty_env_logger::init();

    let request = subscribe(&Context::new())
        .connect("tcp://127.0.0.1:7899")
        .expect("Couldn't connect")
        .subscribe("")
        .for_each(|val| {
            info!("Subscribe: {}", val.as_str().unwrap_or(""));
            Ok(())
        })
        .map_err(|e| {
            error!("Error Subscribing: {}", e);
        });

    tokio::run(request);
}
