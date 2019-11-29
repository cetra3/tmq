use zmq::{self, Context as ZmqContext};

use crate::{comm::Sender, poll::ZmqPoller};

pub fn push(context: &ZmqContext) -> PushBuilder {
    PushBuilder { context }
}

pub struct PushBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> PushBuilder<'a> {
    build_connect!(PUSH, PushBuilderBound);
}

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

pub struct Push {
    inner: Sender,
}
impl_wrapper!(Push, Sender, inner);
impl_wrapper_sink!(Push, inner);
