use zmq::{self, Context as ZmqContext};

use crate::{comm::Receiver, poll::ZmqPoller};

pub fn pull(context: &ZmqContext) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> PullBuilder<'a> {
    build_bind!(PULL, PullBuilderBound);
}

pub struct PullBuilderBound {
    socket: zmq::Socket,
}

impl PullBuilderBound {
    pub fn finish(self) -> Pull {
        Pull {
            inner: Receiver::new(ZmqPoller::from_zmq_socket(self.socket)),
        }
    }
}

pub struct Pull {
    inner: Receiver,
}
impl_wrapper!(Pull, Receiver, inner);
impl_wrapper_stream!(Pull, inner);
impl_buffered!(Pull, inner);
