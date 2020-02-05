use zmq::{self, Context as ZmqContext};

use crate::{
    comm::SenderReceiver, poll::ZmqPoller, socket::AsZmqSocket, FromZmqSocket, Result,
    SocketBuilder,
};

/// Create a builder for a ROUTER socket.
pub fn router(context: &ZmqContext) -> SocketBuilder<Router> {
    SocketBuilder::new(context, zmq::SocketType::ROUTER)
}

/// Asynchronous ROUTER socket.
pub struct Router {
    inner: SenderReceiver,
}

impl FromZmqSocket<Router> for Router {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<Self> {
        Ok(Self {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(socket)?),
        })
    }
}

impl_wrapper!(Router, SenderReceiver, inner);
impl_wrapper_sink!(Router, inner);
impl_wrapper_stream!(Router, inner);
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
