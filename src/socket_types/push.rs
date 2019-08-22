use zmq::{self, Context as ZmqContext};

use crate::{poll::EventedSocket, Multipart};

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

impl_socket!(Push, socket);
impl_sink!(Push, socket, buffer);
