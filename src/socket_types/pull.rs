use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::wrapper::ReceiveWrapper;

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
            inner: ReceiveWrapper::new(ZmqPoller::from_zmq_socket(self.socket)),
        }
    }
}

pub struct Pull {
    inner: ReceiveWrapper,
}
impl_wrapper_deref!(Pull, ReceiveWrapper, inner);
impl_buffered!(Pull, inner);
