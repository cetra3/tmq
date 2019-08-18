use zmq::{self, Context as ZmqContext, SocketType};

use crate::poll::EventedSocket;
use crate::{Result, TmqError, Multipart};
use futures::Sink;
use std::task::{Poll, Context};
use std::pin::Pin;

pub fn push(context: &ZmqContext) -> PushBuilder {
    PushBuilder { context }
}

pub struct PushBuilder<'a> {
    context: &'a ZmqContext,
}

pub struct PushBuilderBounded {
    socket: zmq::Socket,
}

impl<'a> PushBuilder<'a> {
    pub fn bind(self, endpoint: &str) -> Result<PushBuilderBounded> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.bind(endpoint)?;

        Ok(PushBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn connect(self, endpoint: &str) -> Result<PushBuilderBounded> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.connect(endpoint)?;

        Ok(PushBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl PushBuilderBounded {
    pub fn finish(self) -> Push {
        Push {
            socket: EventedSocket::from_zmq_socket(self.socket),
            buffer: None
        }
    }
}

pub struct Push {
    socket: EventedSocket,
    buffer: Option<Multipart>
}

impl Sink<Multipart> for Push {
    type Error = TmqError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        if let Some(data) = self.buffer.take() {
            self.buffer = self.socket.multipart_flush(cx, data)?;
            match self.buffer {
                Some(_) => return Poll::Pending,
                None => {}
            }
        }

        self.socket.multipart_send_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Multipart) -> Result<()> {
        assert_eq!(self.buffer, None);
        self.buffer = Some(item);
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        if let Some(data) = self.buffer.take() {
            self.buffer = self.socket.multipart_flush(cx, data)?;
            match self.buffer {
                Some(_) => return Poll::Pending,
                None => return Poll::Ready(Ok(()))
            }
        }
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        self.poll_flush(cx)
    }
}
