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

impl<'a> DealerBuilder<'a> {
    build_connect!(DEALER, DealerBuilderBound);
    build_bind!(DEALER, DealerBuilderBound);
}

pub struct DealerBuilderBound {
    socket: zmq::Socket,
}

impl DealerBuilderBound {
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

impl_stream!(Dealer, socket);
impl_sink!(Dealer, socket, buffer);
