use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, Sender};

/// Create a builder for a PUB socket.
pub fn publish(context: &ZmqContext) -> PublishBuilder {
    PublishBuilder::new(context)
}

impl_builder!(PUB, PublishBuilder, PublishBuilderBound);

pub struct PublishBuilderBound {
    socket: zmq::Socket,
}

impl PublishBuilderBound {
    pub fn finish(self) -> crate::Result<Publish> {
        Ok(Publish {
            inner: Sender::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous PUB socket.
pub struct Publish {
    inner: Sender,
}
impl_wrapper!(Publish, Sender, inner);
impl_wrapper_sink!(Publish, inner);
