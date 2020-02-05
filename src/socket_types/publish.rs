use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, FromZmqSocket, Sender, SocketBuilder};

/// Create a builder for a PUB socket.
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
