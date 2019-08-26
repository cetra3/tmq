use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::wrapper::SendWrapper;

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
    pub fn finish(self) -> Push {
        Push {
            inner: SendWrapper::new(ZmqPoller::from_zmq_socket(self.socket)),
        }
    }
}

pub struct Push {
    inner: SendWrapper,
}
impl_wrapper_deref!(Push, SendWrapper, inner);
