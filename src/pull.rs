use crate::subscribe::{Sub};

use tokio::reactor::PollEvented2;

use failure::Error;

use crate::socket::MioSocket;
use zmq::{self, Context, SocketType};

/// Allows the `PULL` style socket to be created.  See the `pull` example
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

    pub fn bind(self, endpoint: &str) -> Result<Sub<PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.bind(endpoint)?;

        Ok(Sub {
            socket: PollEvented2::new(socket.into()),
            buffer: None,
        })
    }
}
