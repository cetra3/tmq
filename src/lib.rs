extern crate failure;

extern crate futures;
extern crate mio;
extern crate tokio;
extern crate zmq;

#[macro_use]
extern crate log;

mod publish;
mod pull;
mod push;
mod request;
mod respond;
mod subscribe;

mod poll;
mod socket;

pub use poll::Poller;
pub use socket::MioSocket;

pub use publish::{publish, Pub};
pub use subscribe::{subscribe, Sub, SubMpart};

pub use pull::{pull, Pull};
pub use push::{push, Push};
pub use request::{request, Req};
pub use respond::{respond, Rep, Responder};

pub use zmq::{Context, Message, Result, Socket, SocketType};

pub enum TmqMessage {
    Single(zmq::Message),
    Multipart(Vec<zmq::Message>),
}

impl From<zmq::Message> for TmqMessage {
    fn from(msg: zmq::Message) -> Self {
        TmqMessage::Single(msg)
    }
}

impl From<Vec<zmq::Message>> for TmqMessage {
    fn from(msgs: Vec<zmq::Message>) -> Self {
        TmqMessage::Multipart(msgs)
    }
}
