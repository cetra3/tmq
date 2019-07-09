use crate::publish::Pub;
use crate::TmqMessage;

use failure::Error;

use tokio::reactor::PollEvented2;

use std::collections::VecDeque;

use crate::socket::MioSocket;
use zmq::{self, Context, SocketType};

/// Allows the `PUSH` style socket to be created.  See the `push` example
pub fn push(context: &Context) -> PushBuilder {
    PushBuilder { context }
}

pub struct PushBuilder<'a> {
    context: &'a Context,
}

impl<'a> PushBuilder<'a> {
    pub fn bind<M: Into<TmqMessage>>(
        self,
        endpoint: &str,
    ) -> Result<Pub<M, PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.bind(endpoint)?;

        Ok(Pub {
            socket: PollEvented2::new(socket.into()),
            buffer: VecDeque::new(),
            current: None,
        })
    }

    pub fn connect<M: Into<TmqMessage>>(
        self,
        endpoint: &str,
    ) -> Result<Pub<M, PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PUSH)?;
        socket.connect(endpoint)?;

        Ok(Pub {
            socket: PollEvented2::new(socket.into()),
            buffer: VecDeque::new(),
            current: None,
        })
    }
}
