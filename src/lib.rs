#[macro_use]
extern crate quick_error;

pub use zmq::{Context, Message, Socket, SocketType};

pub use error::TmqError;

pub use crate::message::Multipart;
//pub use crate::publish::{publish, Pub};
pub use crate::pull::pull;

//mod publish;
mod error;
mod pull;
mod message;
//mod push;
//mod request;
//mod respond;
//mod subscribe;

mod socket;
mod poll;

pub type Result<T> = std::result::Result<T, TmqError>;

//pub use crate::push::{push, Push};
//pub use crate::request::{request, Req};
//pub use crate::respond::{respond, Rep, Responder};
//pub use crate::socket::MioSocket;
//pub use crate::subscribe::{subscribe, Sub};
