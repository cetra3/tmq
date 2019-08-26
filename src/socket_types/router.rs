use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::Multipart;

pub fn router(context: &ZmqContext) -> RouterBuilder {
    RouterBuilder { context }
}

pub struct RouterBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> RouterBuilder<'a> {
    build_bind!(ROUTER, RouterBuilderBound);
    build_connect!(ROUTER, RouterBuilderBound);
}

pub struct RouterBuilderBound {
    socket: zmq::Socket,
}

impl RouterBuilderBound {
    pub fn finish(self) -> Router {
        Router {
            socket: ZmqPoller::from_zmq_socket(self.socket),
            buffer: Multipart::default()
        }
    }
}

pub struct Router {
    socket: ZmqPoller,
    buffer: Multipart
}

impl_socket!(Router, socket);
impl_stream!(Router, socket);
impl_sink!(Router, buffer, socket);
