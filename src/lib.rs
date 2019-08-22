#[macro_use]
extern crate quick_error;

#[macro_use]
mod macros;

mod error;
mod message;
mod poll;
mod socket;
mod socket_types;

pub type Result<T> = std::result::Result<T, TmqError>;

/// External re-exports
pub use zmq::{Context, Message};

/// Internal re-exports
pub use crate::message::Multipart;
pub use error::TmqError;

pub use socket_types::*;
