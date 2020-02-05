use crate::FromZmqSocket;
use zmq::{Context, SocketType};

/// Builder which provides [`bind`] and [`connect`] methods to build a corresponding ZMQ socket.
///
/// [`bind`]: struct.SocketBuilder.html#method.bind
/// [`connect`]: struct.SocketBuilder.html#method.connect
pub struct SocketBuilder<'a, T> {
    context: &'a ::zmq::Context,
    socket_type: ::zmq::SocketType,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> SocketBuilder<'a, T>
where
    T: FromZmqSocket<T>,
{
    #[doc(hidden)]
    pub(crate) fn new(context: &'a Context, socket_type: SocketType) -> Self {
        Self {
            context,
            socket_type,
            _phantom: Default::default(),
        }
    }

    /// Connect to a ZMQ endpoint at the given address.
    pub fn connect(self, endpoint: &str) -> crate::Result<T> {
        let socket = self.context.socket(self.socket_type)?;
        socket.connect(endpoint)?;
        T::from_zmq_socket(socket)
    }

    /// Bind to a ZMQ endpoint at the given address.
    pub fn bind(self, endpoint: &str) -> crate::Result<T> {
        let socket = self.context.socket(self.socket_type)?;
        socket.bind(endpoint)?;
        T::from_zmq_socket(socket)
    }
}
