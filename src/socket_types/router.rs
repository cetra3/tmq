use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::{socket::AsZmqSocket, Multipart, Result};

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
            buffer: Multipart::default(),
        }
    }
}

pub struct Router {
    socket: ZmqPoller,
    buffer: Multipart,
}

impl Router {
    pub fn is_router_mandatory(&self) -> Result<bool> {
        Ok(self.get_socket().is_router_mandatory()?)
    }
    pub fn set_router_mandatory(&self, value: bool) -> Result<()> {
        self.get_socket().set_router_mandatory(value)?;
        Ok(())
    }

    pub fn is_router_handover(&self) -> Result<bool> {
        Ok(self.get_socket().is_router_handover()?)
    }
    pub fn set_router_handover(&self, value: bool) -> Result<()> {
        self.get_socket().set_router_handover(value)?;
        Ok(())
    }
}

impl_socket!(Router, socket);
impl_stream!(Router, socket);
impl_sink!(Router, buffer, socket);
