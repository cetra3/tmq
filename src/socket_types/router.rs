use zmq::{self, Context as ZmqContext};

use crate::{poll::EventedSocket, Multipart};

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
            socket: EventedSocket::from_zmq_socket(self.socket),
        }
    }
}

pub struct Router {
    socket: EventedSocket,
}

impl_socket!(Router, socket);
impl_stream!(Router, socket);
impl_sink!(Router, socket);
