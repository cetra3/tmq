use zmq::{self, Context as ZmqContext};

use crate::{comm::SenderReceiver, poll::ZmqPoller};

pub fn reply(context: &ZmqContext) -> RepBuilder {
    RepBuilder { context }
}

pub struct RepBuilder<'a> {
    context: &'a ZmqContext,
}

impl<'a> RepBuilder<'a> {
    build_bind!(REP, RepBuilderBound);
}

pub struct RepBuilderBound {
    socket: zmq::Socket,
}

impl RepBuilderBound {
    pub fn finish(self) -> crate::Result<Reply> {
        Ok(Reply {
            inner: SenderReceiver::new(ZmqPoller::from_zmq_socket(self.socket)?),
        })
    }
}

pub struct Reply {
    inner: SenderReceiver,
}
impl_wrapper!(Reply, SenderReceiver, inner);
impl_wrapper_sink!(Reply, inner);
impl_wrapper_stream!(Reply, inner);

