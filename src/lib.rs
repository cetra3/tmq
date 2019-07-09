//! # TMQ - Rust ZeroMQ bindings for Tokio
//! 
//! This crate bridges Tokio and ZeroMQ to allow for ZeroMQ in the async world.
//! 
//! Currently a WIP
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

use std::ops::{Deref, DerefMut};
use std::fmt;

pub use publish::publish;
pub use subscribe::subscribe;

pub use pull::pull;
pub use push::push;
pub use request::request;
pub use respond::{respond, Responder};

pub use zmq::{Context};

/// The Main Message Type for `tmq`.  This represents one or more messages received or sent.
/// 
/// You can create a TmqMessage from a variety of sources including, `&[u8]`, `String`, `&str`, `zmq::Message` or their `Vec` counterparts for multipart messages.
/// 
/// ```rust
/// # use tmq::*;
/// # fn main() {
/// let message = TmqMessage::from("Message");
/// 
/// let multipart_message = TmqMessage::from(vec!["Message1", "Message2"]);
/// # }
/// ```
/// 
pub struct TmqMessage {
    pub msgs: Vec<zmq::Message>,
}
impl TmqMessage {
    /// If there is more than one message, this function returns true
    pub fn has_more(&self) -> bool {
        self.msgs.len() > 1
    }
}

impl fmt::Display for TmqMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msgs.iter()
                    .filter_map(|msg| msg.as_str())
                    .collect::<Vec<&str>>()
                    .join(", "))
    }
}

impl From<zmq::Message> for TmqMessage {
    fn from(msg: zmq::Message) -> Self {
        TmqMessage { msgs: vec![msg] }
    }
}

impl From<Vec<zmq::Message>> for TmqMessage {
    fn from(msgs: Vec<zmq::Message>) -> Self {
        TmqMessage { msgs }
    }
}

impl From<Vec<&str>> for TmqMessage {
    fn from(msgs: Vec<&str>) -> Self {
        TmqMessage { msgs: msgs.iter().map(|msg| msg.into()).collect() }
    }
}

impl From<Vec<&String>> for TmqMessage {
    fn from(msgs: Vec<&String>) -> Self {
        TmqMessage { msgs: msgs.iter().map(|msg| msg.into()).collect() }
    }
}

impl From<&str> for TmqMessage {
    fn from(msg: &str) -> Self {
        TmqMessage { msgs: vec!(msg.into())}
    }
}

impl From<&[u8]> for TmqMessage {
    fn from(msg: &[u8]) -> Self {
        TmqMessage { msgs: vec!(msg.into())}
    }
}

impl From<&String> for TmqMessage {
    fn from(msg: &String) -> Self {
        TmqMessage { msgs: vec!(msg.into())}
    }
}


impl From<Vec<u8>> for TmqMessage {
    fn from(msg: Vec<u8>) -> Self {
        TmqMessage { msgs: vec!(msg.into())}
    }
}

impl From<Vec<Vec<u8>>> for TmqMessage {
    fn from(msgs: Vec<Vec<u8>>) -> Self {
        TmqMessage { msgs: msgs.iter().map(|msg| msg.into()).collect() }
    }
}


impl From<Box<[u8]>> for TmqMessage {
    fn from(msg: Box<[u8]>) -> Self {
        TmqMessage { msgs: vec!(msg.into())}
    }
}

impl From<TmqMessage> for Vec<zmq::Message> {
    fn from(msg: TmqMessage) -> Self {
        msg.msgs
    }
}

impl From<TmqMessage> for zmq::Message {
    fn from(mut msg: TmqMessage) -> Self {
        msg.msgs.pop().unwrap_or_else(|| zmq::Message::new())
    }
}

impl Default for TmqMessage {
    fn default() -> Self {
        TmqMessage {
            msgs: vec![zmq::Message::new()],
        }
    }
}

impl Deref for TmqMessage {
    type Target = Vec<zmq::Message>;

    fn deref(&self) -> &Self::Target {
        &self.msgs
    }
}

impl DerefMut for TmqMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.msgs
    }
}
