use zmq::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller};

/// Create a builder for a DEALER socket.
pub fn dealer(context: &ZmqContext) -> DealerBuilder {
    DealerBuilder::new(context)
}

impl_builder!(DEALER, DealerBuilder, DealerBuilderBound);

pub struct DealerBuilderBound {
    socket: zmq::Socket,
}

impl DealerBuilderBound {
    pub fn finish(self) -> crate::Result<Dealer> {
        Ok(Dealer {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

/// Asynchronous DEALER Socket.
pub struct Dealer {
    inner: SenderReceiver,
}
impl_wrapper!(Dealer, SenderReceiver, inner);
impl_wrapper_sink!(Dealer, inner);
impl_wrapper_stream!(Dealer, inner);
impl_split!(Dealer, inner);
