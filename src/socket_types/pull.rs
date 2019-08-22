use futures::Stream;
use zmq::{self, Context as ZmqContext, SocketType};

use crate::poll::EventedSocket;
use crate::Multipart;
use crate::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn pull(context: &ZmqContext) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> PullBuilder<'a> {
    build_bind!(PULL, PullBuilderBound);
}

pub struct PullBuilderBound {
    socket: zmq::Socket,
}

impl PullBuilderBound {
    pub fn finish(self) -> Pull {
        Pull {
            socket: EventedSocket::from_zmq_socket(self.socket),
        }
    }
}

pub struct Pull {
    socket: EventedSocket,
}

impl_stream!(Pull, socket);
