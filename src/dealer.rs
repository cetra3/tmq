use futures::{Sink, Stream};
use zmq::{self, Context as ZmqContext, SocketType};

use crate::poll::EventedSocket;
use crate::Result;
use crate::{Multipart, TmqError};
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn dealer(context: &ZmqContext) -> DealerBuilder {
    DealerBuilder { context }
}

pub struct DealerBuilder<'a> {
    context: &'a ZmqContext,
}

pub struct DealerBuilderBounded {
    socket: zmq::Socket,
}

impl<'a> DealerBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<DealerBuilderBounded> {
        let socket = self.context.socket(SocketType::DEALER)?;
        socket.connect(endpoint)?;

        Ok(DealerBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<DealerBuilderBounded> {
        let socket = self.context.socket(SocketType::DEALER)?;
        socket.bind(endpoint)?;

        Ok(DealerBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl DealerBuilderBounded {
    pub fn finish(self) -> Dealer {
        Dealer {
            socket: EventedSocket::from_zmq_socket(self.socket),
            buffer: None,
        }
    }
}

pub struct Dealer {
    socket: EventedSocket,
    buffer: Option<Multipart>,
}

impl Stream for Dealer {
    type Item = Result<Multipart>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.socket.multipart_recv(cx)
    }
}

impl Sink<Multipart> for Dealer {
    type Error = TmqError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let buf = self.buffer.take();
        let (poll, buffer) = self.socket.multipart_flush(cx, buf, true);
        self.buffer = buffer;
        poll
    }

    fn start_send(mut self: Pin<&mut Self>, item: Multipart) -> Result<()> {
        assert_eq!(self.buffer, None);
        self.buffer = Some(item);
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let buf = self.buffer.take();
        let (poll, buffer) = self.socket.multipart_flush(cx, buf, false);
        self.buffer = buffer;
        poll
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        self.poll_flush(cx)
    }
}
