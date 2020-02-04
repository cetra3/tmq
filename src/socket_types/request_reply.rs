use crate::{poll::ZmqPoller, Multipart};
use zmq::{self, Context as ZmqContext};

/// Create a builder for a Request socket
pub fn request(context: &ZmqContext) -> RequestBuilder {
    RequestBuilder::new(context)
}

impl_builder!(REQ, RequestBuilder, RequestBuilderBound);

pub struct RequestBuilderBound {
    socket: zmq::Socket,
}

impl RequestBuilderBound {
    pub fn finish(self) -> crate::Result<RequestSender> {
        Ok(RequestSender {
            poller: ZmqPoller::from_zmq_socket(self.socket)?,
        })
    }
}

pub struct RequestSender {
    poller: ZmqPoller,
}

impl crate::socket::AsZmqSocket for RequestSender {
    #[inline]
    fn get_socket(&self) -> &zmq::Socket {
        self.poller.get_socket()
    }
}

impl RequestSender {
    pub async fn send(self, mut msg: Multipart) -> crate::Result<RequestReceiver> {
        futures::future::poll_fn(|cx| self.poller.multipart_flush(cx, &mut msg)).await?;
        Ok(RequestReceiver {
            poller: self.poller,
        })
    }
}

pub fn reply(context: &ZmqContext) -> ReplyBuilder {
    ReplyBuilder::new(context)
}

impl_builder!(REP, ReplyBuilder, ReplyBuilderBound);

pub struct ReplyBuilderBound {
    socket: zmq::Socket,
}

impl ReplyBuilderBound {
    pub fn finish(self) -> crate::Result<RequestReceiver> {
        Ok(RequestReceiver {
            poller: ZmqPoller::from_zmq_socket(self.socket)?,
        })
    }
}

pub struct RequestReceiver {
    poller: ZmqPoller,
}

impl crate::socket::AsZmqSocket for RequestReceiver {
    #[inline]
    fn get_socket(&self) -> &zmq::Socket {
        self.poller.get_socket()
    }
}

impl RequestReceiver {
    pub async fn recv(self) -> crate::Result<(Multipart, RequestSender)> {
        let msg = futures::future::poll_fn(|cx| self.poller.multipart_recv(cx)).await?;
        Ok((
            msg,
            RequestSender {
                poller: self.poller,
            },
        ))
    }
}
