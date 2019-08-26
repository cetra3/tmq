use zmq::{self, Context as ZmqContext};

use crate::poll::ZmqPoller;
use crate::wrapper::SendReceiveWrapper;

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
            inner: SendReceiveWrapper::new(ZmqPoller::from_zmq_socket(self.socket)),
        }
    }
}

pub struct Dealer {
    inner: SendReceiveWrapper,
}
impl_wrapper_deref!(Dealer, SendReceiveWrapper, inner);
impl_split!(Dealer, inner);
