use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Sink;
use zmq::{self, Context as ZmqContext, SocketType};

use crate::poll::EventedSocket;
use crate::{Multipart, Result};

pub fn push(context: &ZmqContext) -> PushBuilder {
    PushBuilder { context }
}

pub struct PushBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> PushBuilder<'a> {
    build_connect!(PUSH, PushBuilderBound);
}

pub struct PushBuilderBound {
    socket: zmq::Socket,
}

impl PushBuilderBound {
    pub fn finish(self) -> Push {
        Push {
            socket: EventedSocket::from_zmq_socket(self.socket),
            buffer: None,
        }
    }
}

pub struct Push {
    socket: EventedSocket,
    buffer: Option<Multipart>,
}

impl_sink!(Push, socket, buffer);
