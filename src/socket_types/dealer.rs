use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::Multipart;

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
            socket: ZmqPoller::from_zmq_socket(self.socket),
            buffer: Multipart::default(),
        }
    }
}

pub struct Dealer {
    socket: ZmqPoller,
    buffer: Multipart,
}

impl_socket!(Dealer, socket);
impl_stream!(Dealer, socket);
impl_sink!(Dealer, buffer, socket);
impl_split!(Dealer, DealerReadHalf, DealerWriteHalf, socket, buffer);
