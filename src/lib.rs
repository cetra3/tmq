#[macro_use]
extern crate quick_error;

pub type Result<T> = std::result::Result<T, TmqError>;

/// External re-exports
pub use zmq::{Context, Message};

pub use error::TmqError;
/// Internal re-exports
pub use message::Multipart;

pub use socket::SocketExt;
pub use socket_types::*;

/// Crate re-exports
pub(crate) use comm::*;

#[macro_use]
mod macros;

mod comm;
mod error;
mod message;
mod poll;
mod socket;
mod socket_types;
