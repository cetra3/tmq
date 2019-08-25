use zmq::{self, Context as ZmqContext};

use crate::poll::EventedSocket;

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
        }
    }
}

pub struct Dealer {
    socket: EventedSocket,
}

impl_socket!(Dealer, socket);
impl_stream!(Dealer, socket);
impl_sink!(Dealer, socket);
