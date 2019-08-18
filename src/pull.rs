use futures::Stream;
use tokio::reactor::PollEvented;
use zmq::{self, Context as ZmqContext, SocketType};

use crate::Multipart;
use crate::poll::EventedSocket;
use crate::Result;
use crate::socket::SocketWrapper;
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn pull(context: &ZmqContext) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a ZmqContext,
}

pub struct PullBuilderBounded {
    socket: zmq::Socket,
}

impl<'a> PullBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<PullBuilderBounded> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<PullBuilderBounded> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.bind(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl PullBuilderBounded {
    pub fn finish(self) -> Pull {
        Pull {
            socket: EventedSocket(PollEvented::new(SocketWrapper::new(self.socket)))
        }
    }
}

pub struct Pull {
    socket: EventedSocket,
}

impl Stream for Pull {
    type Item = Result<Multipart>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.socket.poll_receive_multipart(cx)
    }
}
