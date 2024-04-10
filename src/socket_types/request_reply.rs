use std::pin::Pin;

use crate::{poll::ZmqPoller, FromZmqSocket, Multipart, SocketBuilder};
use zmq::Context as ZmqContext;

/// Create a builder for a REQ socket
pub fn request(context: &ZmqContext) -> SocketBuilder<RequestSender> {
    SocketBuilder::new(context, zmq::SocketType::REQ)
}

/// A REQ Socket returned from the `request` fn
pub struct RequestSender {
    inner: ZmqPoller,
}

impl FromZmqSocket<RequestSender> for RequestSender {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<Self> {
        Ok(Self {
            inner: ZmqPoller::from_zmq_socket(socket)?,
        })
    }
}

impl_as_socket!(RequestSender, inner);

impl RequestSender {
    /// Send a multipart message and return a `RequestReceiver`
    pub async fn send(mut self, mut msg: Multipart) -> crate::Result<RequestReceiver> {
        futures::future::poll_fn(|cx| Pin::new(&mut self.inner).multipart_flush(cx, &mut msg))
            .await?;
        Ok(RequestReceiver { inner: self.inner })
    }
}

/// Create a builder for a REP socket
pub fn reply(context: &ZmqContext) -> SocketBuilder<RequestReceiver> {
    SocketBuilder::new(context, zmq::SocketType::REP)
}
/// A `RequestReceiver` is returned After sending a message
pub struct RequestReceiver {
    inner: ZmqPoller,
}

impl FromZmqSocket<RequestReceiver> for RequestReceiver {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<Self> {
        Ok(Self {
            inner: ZmqPoller::from_zmq_socket(socket)?,
        })
    }
}

impl_as_socket!(RequestReceiver, inner);

impl RequestReceiver {
    /// Receive a multipart message and return a `RequestSender`
    pub async fn recv(mut self) -> crate::Result<(Multipart, RequestSender)> {
        let msg =
            futures::future::poll_fn(|cx| Pin::new(&mut self.inner).multipart_recv(cx)).await?;
        Ok((msg, RequestSender { inner: self.inner }))
    }
}
