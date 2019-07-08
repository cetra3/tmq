use crate::subscribe::{Sub, SubMpart};

pub type Pull<P> = Sub<P>;

use tokio::reactor::PollEvented2;

use failure::Error;

use crate::socket::MioSocket;
use zmq::{self, Context, SocketType};

pub fn pull(context: &Context) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a Context,
}

impl<'a> PullBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<Sub<PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(Sub {
            socket: PollEvented2::new(socket.into()),
            buffer: None,
        })
    }

    pub fn connect_mpart(self, endpoint: &str) -> Result<SubMpart<PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(SubMpart {
            socket: PollEvented2::new(socket.into()),
            buffer: None,
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<Sub<PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.bind(endpoint)?;

        Ok(Sub {
            socket: PollEvented2::new(socket.into()),
            buffer: None,
        })
    }

    pub fn bind_mpart(self, endpoint: &str) -> Result<SubMpart<PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(SubMpart {
            socket: PollEvented2::new(socket.into()),
            buffer: None,
        })
    }
}
