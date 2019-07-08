extern crate futures;
extern crate pretty_env_logger;
extern crate tmq;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate failure;

use failure::Error;
use futures::future::ok;
use futures::future::FutureResult;
use futures::Future;

use tmq::*;

use std::env;

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "response=DEBUG");
    }

    pretty_env_logger::init();

    let responder = respond(&Context::new())
        .bind("tcp://127.0.0.1:7899")
        .expect("Couldn't bind address")
        .with(echo)
        .map_err(|err| {
            error!("Error from server:{}", err);
        });

    tokio::run(responder);
}

//You can use a struct to respond by implementing the `Responder` trait
pub struct EchoResponder {}

impl Responder<TmqMessage> for EchoResponder {
    type Output = FutureResult<TmqMessage, Error>;

    fn respond(&mut self, msg: TmqMessage) -> Self::Output {
        return Ok(msg.into()).into();
    }
}

//Or you can use a free-floating function
#[allow(unused)]
fn echo(msg: Message) -> impl Future<Item = Message, Error = Error> {
    return ok(msg);
}
