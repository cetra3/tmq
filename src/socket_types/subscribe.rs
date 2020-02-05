use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, socket::AsZmqSocket, FromZmqSocket, Receiver, SocketBuilder};

/// Create a builder for a SUB socket.
pub fn subscribe(context: &ZmqContext) -> SocketBuilder<SubscribeWithoutTopic> {
    SocketBuilder::new(context, zmq::SocketType::SUB)
}

/// SUB socket which is already bound or connected, but isn't yet subscribed to a topic.
pub struct SubscribeWithoutTopic {
    socket: zmq::Socket,
}

impl FromZmqSocket<SubscribeWithoutTopic> for SubscribeWithoutTopic {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<Self> {
        Ok(Self { socket })
    }
}

impl SubscribeWithoutTopic {
    /// Finishes creating the SUB socket by subscribing to the given topic.
    pub fn subscribe(self, topic: &[u8]) -> crate::Result<Subscribe> {
        self.socket.set_subscribe(topic)?;
        Ok(Subscribe {
            inner: Receiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous SUB socket.
pub struct Subscribe {
    inner: Receiver,
}
impl_wrapper!(Subscribe, Receiver, inner);
impl_wrapper_stream!(Subscribe, inner);

impl Subscribe {
    /// Adds another topic to this subscriber.
    /// This doesn't remove the previously added topics.
    pub fn subscribe(&mut self, topic: &[u8]) -> crate::Result<()> {
        self.get_socket().set_subscribe(topic)?;
        Ok(())
    }
}
