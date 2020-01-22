use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, socket::AsZmqSocket, Receiver};

/// Create a builder for a SUB socket.
pub fn subscribe(context: &ZmqContext) -> SubscribeBuilder {
    SubscribeBuilder { context }
}

pub struct SubscribeBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> SubscribeBuilder<'a> {
    build_connect!(SUB, SubscribeBuilderBound);
}

pub struct SubscribeBuilderBound {
    socket: zmq::Socket,
}

impl SubscribeBuilderBound {
    pub fn subscribe(self, address: &[u8]) -> crate::Result<Subscribe> {
        self.socket.set_subscribe(address)?;
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
    pub fn subscribe(&mut self, address: &[u8]) -> crate::Result<()> {
        self.get_socket().set_subscribe(address)?;
        Ok(())
    }
}
