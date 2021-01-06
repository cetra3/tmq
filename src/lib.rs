#![deny(missing_docs)]

//! # TMQ - Rust ZeroMQ bindings for Tokio
//!
//! This crate bridges Tokio and ZeroMQ to allow for ZeroMQ in the async world.
//!
//! ## Currently Implemented Sockets
//!
//! * Request/Reply
//! * Publish/Subscribe
//! * Dealer/Router
//! * Push/Pull
//! ## Usage
//!
//! Usage is made to be simple, but opinionated.   See the [`examples/`](https://github.com/cetra3/tmq/tree/master/examples) Directory for some examples.
//!
//! ### Publish Example
//!
//! To publish messages to all connected subscribers, you can use the `publish` function:
//!
//! ```rust,no_run
//! use tmq::{publish, Context, Result};
//!
//! use futures::SinkExt;
//! use log::info;
//! use std::env;
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!
//!     let mut socket = publish(&Context::new()).bind("tcp://127.0.0.1:7899")?;
//!
//!     let mut i = 0;
//!
//!     loop {
//!         i += 1;
//!
//!         socket
//!             .send(vec!["topic", &format!("Broadcast #{}", i)])
//!             .await?;
//!
//!         sleep(Duration::from_secs(1)).await;
//!     }
//! }
//! ```

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
