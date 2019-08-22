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
            buffer: None,
        }
    }
}

pub struct Push {
    socket: EventedSocket,
    buffer: Option<Multipart>,
}

impl_sink!(Push, socket, buffer);
