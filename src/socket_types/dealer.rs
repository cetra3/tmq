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

impl_stream!(Dealer, socket);
impl_sink!(Dealer, socket, buffer);
