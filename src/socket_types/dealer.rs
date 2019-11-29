use zmq::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller};

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
    pub fn finish(self) -> crate::Result<Dealer> {
        Ok(Dealer {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

pub struct Dealer {
    inner: SenderReceiver,
}
impl_wrapper!(Dealer, SenderReceiver, inner);
impl_wrapper_sink!(Dealer, inner);
impl_wrapper_stream!(Dealer, inner);
impl_split!(Dealer, inner);
