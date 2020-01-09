use zmq::{self, Context as ZmqContext};

use crate::{poll::ZmqPoller, Receiver};

pub fn subscribe(context: &ZmqContext) -> SubscribeBuilder {
    SubscribeBuilder { context }
}

pub struct SubscribeBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> SubscribeBuilder<'a> {
    build_connect!(SUB, SubscribeBuilderBound);
}

pub struct SubscribeBuilderBound {
    socket: zmq::Socket,
}

impl SubscribeBuilderBound {
    pub fn subscribe(self, address: &[u8]) -> crate::Result<Subscribe> {
        self.socket.set_subscribe(address)?;
        Ok(Subscribe {
            inner: Receiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

pub struct Subscribe {
    inner: Receiver,
}
impl_wrapper!(Subscribe, Receiver, inner);
impl_wrapper_stream!(Subscribe, inner);
