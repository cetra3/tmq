use zmq::Context as ZmqContext;

use crate::{poll::ZmqPoller, FromZmqSocket, Sender, SocketBuilder};

/// Create a builder for a PUB socket.
///
/// ## Usage Example
///
/// ```rust,no_run
/// use tmq::{publish, Context, Result};
///
/// use futures::SinkExt;
/// use log::info;
/// use std::env;
/// use std::time::Duration;
/// use tokio::time::sleep;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///
///     let mut socket = publish(&Context::new()).bind("tcp://127.0.0.1:7899")?;
///
///     let mut i = 0;
///
///     loop {
///         i += 1;
///
///         socket
///             .send(vec!["topic", &format!("Broadcast #{}", i)])
///             .await?;
///
///         sleep(Duration::from_secs(1)).await;
///     }
/// }
/// ```
pub fn publish(context: &ZmqContext) -> SocketBuilder<Publish> {
    SocketBuilder::new(context, zmq::SocketType::PUB)
}

/// Asynchronous PUB socket.
pub struct Publish {
    inner: Sender,
}

impl FromZmqSocket<Publish> for Publish {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<Self> {
        Ok(Self {
            inner: Sender::new(ZmqPoller::from_zmq_socket(socket)?),
        })
    }
}

impl_wrapper!(Publish, Sender, inner);
impl_wrapper_sink!(Publish, inner);
