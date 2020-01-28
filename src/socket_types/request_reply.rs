use crate::TmqError;
use crate::{poll::ZmqPoller, Multipart};
use futures::ready;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
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
    pub fn send(self, msg: &mut Multipart) -> RequestSend {
        RequestSend {
            poller: Some(self.poller),
            msg,
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
    pub fn recv(self) -> RequestRecv {
        RequestRecv {
            poller: Some(self.poller),
        }
    }
}

pub struct RequestSend<'msg> {
    poller: Option<ZmqPoller>,
    msg: &'msg mut Multipart,
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

impl<'msg> Future for RequestSend<'msg> {
    type Output = std::result::Result<RequestReceiver, SendError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let Self {
            ref mut poller,
            ref mut msg,
        } = self.get_mut();
        match ready!(poller.as_ref().unwrap().multipart_flush(cx, msg)) {
            Err(err) => Poll::Ready(Err(SendError {
                err,
                req: RequestSender {
                    poller: poller.take().unwrap(),
                },
            })),
            Ok(()) => Poll::Ready(Ok(RequestReceiver {
                poller: poller.take().unwrap(),
            })),
        }
    }
}

pub struct RequestRecv {
    poller: Option<ZmqPoller>,
}

impl Future for RequestRecv {
    type Output = std::result::Result<(Multipart, RequestSender), ReceiveError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let Self { ref mut poller } = self.get_mut();
        // .unwrap() looks ok because I think the Option<Result<>> is rendudant
        // and the Option<> part is always Some()
        match ready!(poller.as_ref().unwrap().multipart_recv(cx)) {
            Ok(msg) => Poll::Ready(Ok((
                msg,
                RequestSender {
                    poller: poller.take().unwrap(),
                },
            ))),
            Err(err) => Poll::Ready(Err(ReceiveError {
                err,
                recv: RequestReceiver {
                    poller: poller.take().unwrap(),
                },
            })),
        }
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
