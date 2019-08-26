use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::wrapper::SendReceiveWrapper;
use crate::{socket::AsZmqSocket, Result};

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
            inner: SendReceiveWrapper::new(ZmqPoller::from_zmq_socket(self.socket)),
        }
    }
}

pub struct Router {
    inner: SendReceiveWrapper,
}
impl_wrapper_deref!(Router, SendReceiveWrapper, inner);
impl_split!(Router, inner);

impl Router {
    pub fn is_router_mandatory(&self) -> Result<bool> {
        Ok(self.inner.get_socket().is_router_mandatory()?)
    }
    pub fn set_router_mandatory(&self, value: bool) -> Result<()> {
        self.inner.get_socket().set_router_mandatory(value)?;
        Ok(())
    }

    pub fn is_router_handover(&self) -> Result<bool> {
        Ok(self.inner.get_socket().is_router_handover()?)
    }
    pub fn set_router_handover(&self, value: bool) -> Result<()> {
        self.inner.get_socket().set_router_handover(value)?;
        Ok(())
    }
}
