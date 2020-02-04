use zmq::{self, Context as ZmqContext};

use crate::{comm::Sender, poll::ZmqPoller};

/// Create a builder for a PUSH socket.
pub fn push(context: &ZmqContext) -> PushBuilder {
    PushBuilder::new(context)
}

impl_builder!(PUSH, PushBuilder, PushBuilderBound);

pub struct PushBuilderBound {
    socket: zmq::Socket,
}

impl PushBuilderBound {
    pub fn finish(self) -> crate::Result<Push> {
        Ok(Push {
            inner: Sender::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous PUSH socket.
pub struct Push {
    inner: Sender,
}
impl_wrapper!(Push, Sender, inner);
impl_wrapper_sink!(Push, inner);
