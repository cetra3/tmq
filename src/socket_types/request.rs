use zmq::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller};

pub fn request(context: &ZmqContext) -> ReqBuilder {
    ReqBuilder { context }
}

pub struct ReqBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> ReqBuilder<'a> {
    build_connect!(REQ, ReqBuilderBound);
}

pub struct ReqBuilderBound {
    socket: zmq::Socket,
}

impl ReqBuilderBound {
    pub fn finish(self) -> crate::Result<Request> {
        Ok(Request {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

pub struct Request {
    inner: SenderReceiver,
}
impl_wrapper!(Request, SenderReceiver, inner);
impl_wrapper_sink!(Request, inner);
impl_wrapper_stream!(Request, inner);

