use zmq::Context as ZmqContext;

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

impl Router {
    /// Accessor for the `ZMQ_ROUTER_MANDATORY` option.
    pub fn is_router_mandatory(&self) -> Result<bool> {
        Ok(self.get_socket().is_router_mandatory()?)
    }

    /// Setter for the `ZMQ_ROUTER_MANDATORY` option.
    pub fn set_router_mandatory(&self, value: bool) -> Result<()> {
        self.get_socket().set_router_mandatory(value)?;
        Ok(())
    }

    /// Accessor for the `ZMQ_ROUTER_HANDOVER` option.
    pub fn is_router_handover(&self) -> Result<bool> {
        Ok(self.get_socket().is_router_handover()?)
    }

    /// Setter for the `ZMQ_ROUTER_HANDOVER` option.
    pub fn set_router_handover(&self, value: bool) -> Result<()> {
        self.get_socket().set_router_handover(value)?;
        Ok(())
    }
}
