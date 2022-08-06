use zmq2::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller, FromZmqSocket, SocketBuilder};

/// Create a builder for a PAIR socket.
pub fn pair(context: &ZmqContext) -> SocketBuilder<Pair> {
    SocketBuilder::new(context, zmq2::SocketType::PAIR)
}

/// Asynchronous PAIR Socket.
pub struct Pair {
    inner: SenderReceiver,
}

impl FromZmqSocket<Pair> for Pair {
    fn from_zmq_socket(socket: zmq2::Socket) -> crate::Result<Self> {
        Ok(Self {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(socket)?),
        })
    }
}

impl_wrapper!(Pair, SenderReceiver, inner);
impl_wrapper_sink!(Pair, inner);
impl_wrapper_stream!(Pair, inner);
