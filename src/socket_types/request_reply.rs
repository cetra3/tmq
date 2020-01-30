use crate::TmqError;
use crate::{poll::ZmqPoller, Multipart};
use thiserror::Error;
use zmq::{self, Context as ZmqContext};

pub fn request(context: &ZmqContext) -> ReqBuilder {
    ReqBuilder { context }
}

pub struct ReqBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> ReqBuilder<'a> {
    build_connect!(REQ, ReqBuilderBound);
    build_bind!(REQ, ReqBuilderBound);
}

pub struct ReqBuilderBound {
    socket: zmq::Socket,
}

impl ReqBuilderBound {
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
    pub async fn send(self, mut msg: Multipart) -> std::result::Result<RequestReceiver, SendError> {
        match futures::future::poll_fn(|cx| self.poller.multipart_flush(cx, &mut msg)).await {
            Ok(()) => Ok(RequestReceiver {
                poller: self.poller,
            }),
            Err(err) => Err(SendError { err, req: self }),
        }
    }
}

pub fn reply(context: &ZmqContext) -> RepBuilder {
    RepBuilder { context }
}

pub struct RepBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> RepBuilder<'a> {
    build_connect!(REP, RepBuilderBound);
    build_bind!(REP, RepBuilderBound);
}

pub struct RepBuilderBound {
    socket: zmq::Socket,
}

impl RepBuilderBound {
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
    pub async fn recv(self) -> std::result::Result<(Multipart, RequestSender), ReceiveError> {
        match futures::future::poll_fn(|cx| self.poller.multipart_recv(cx)).await {
            Ok(msg) => Ok((
                msg,
                RequestSender {
                    poller: self.poller,
                },
            )),
            Err(err) => Err(ReceiveError { err, recv: self }),
        }
    }
}

#[derive(Error)]
#[error("{err}")]
pub struct SendError {
    err: TmqError,
    req: RequestSender,
}

impl std::fmt::Debug for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::convert::From<SendError> for TmqError {
    fn from(r: SendError) -> TmqError {
        r.err
    }
}

#[derive(Error)]
#[error("{err}")]
pub struct ReceiveError {
    err: TmqError,
    recv: RequestReceiver,
}

impl std::fmt::Debug for ReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::convert::From<ReceiveError> for TmqError {
    fn from(r: ReceiveError) -> TmqError {
        r.err
    }
}
