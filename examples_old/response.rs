use failure::Error;
use futures::future::ok;
use futures::future::FutureResult;
use futures::Future;

use tmq::*;

use log::{error, info};
use std::env;

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "response=DEBUG");
    }

    pretty_env_logger::init();

    let responder = respond(&Context::new())
        .bind("tcp://127.0.0.1:7899")
        .expect("Couldn't bind address")
        .with(|msg: Message| {
            info!("Request: {}", msg.as_str().unwrap_or(""));
            Ok(msg)
        })
        .map_err(|err| {
            error!("Error from server:{}", err);
        });

    tokio::run(responder);
}

//You can use a struct to respond by implementing the `Responder` trait
pub struct EchoResponder {}

impl Responder for EchoResponder {
    type Output = FutureResult<Message, Error>;

    fn respond(&mut self, msg: Message) -> Self::Output {
        return Ok(msg).into();
    }
}

//Or you can use a free-floating function
fn echo(msg: Message) -> impl Future<Item = Message, Error = Error> {
    return ok(msg);
}
