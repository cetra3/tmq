/// Shortcut for [Result<T, tmq::TmqError>](std::result::Result).
pub type Result<T> = std::result::Result<T, TmqError>;

/// External re-exports
pub use zmq::{Context, Message};

/// Internal re-exports
pub use error::TmqError;
pub use message::Multipart;
pub use socket::{SocketExt, AsZmqSocket};
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
