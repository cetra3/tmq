#[macro_use]
extern crate quick_error;

pub use zmq::{Context, Message, Socket, SocketType};

pub use error::TmqError;

pub use crate::dealer::dealer;
pub use crate::message::Multipart;
pub use crate::pull::pull;
pub use crate::push::push;

mod dealer;
mod pull;
mod push;

//mod publish;
mod error;
mod message;
mod poll;
mod socket;
//mod request;
//mod respond;
//mod subscribe;

pub type Result<T> = std::result::Result<T, TmqError>;

//pub use crate::request::{request, Req};
//pub use crate::respond::{respond, Rep, Responder};
//pub use crate::socket::MioSocket;
//pub use crate::subscribe::{subscribe, Sub};
