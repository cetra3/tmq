use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;

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
            socket: ZmqPoller::from_zmq_socket(self.socket),
        }
    }
}

pub struct Pull {
    socket: ZmqPoller,
}

impl_socket!(Pull, socket);
impl_stream!(Pull, socket);
