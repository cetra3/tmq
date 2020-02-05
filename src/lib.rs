/// Shortcut for [`Result<T, tmq::TmqError>`].
pub type Result<T> = std::result::Result<T, TmqError>;

pub use zmq::{Context, Message};

/// Internal re-exports
pub use error::TmqError;
pub use message::Multipart;
pub use socket::{AsZmqSocket, SocketExt};
pub use socket_builder::SocketBuilder;
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
mod socket_builder;
mod socket_types;
