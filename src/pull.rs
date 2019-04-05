use futures::{Async, Poll, Stream};

use tokio::reactor::PollEvented2;

use failure::Error;

use zmq::{self, Context, SocketType};

use poll::Poller;
use socket::MioSocket;

pub fn pull(context: &Context) -> PullBuilder {
    PullBuilder { context }
}

pub struct PullBuilder<'a> {
    context: &'a Context,
}

pub struct PullBuilderBounded {
    socket: MioSocket,
}

impl<'a> PullBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<PullBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.connect(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<PullBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::PULL)?;
        socket.bind(endpoint)?;

        Ok(PullBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl PullBuilderBounded {
    pub fn finish(self) -> Pull<PollEvented2<MioSocket>> {
        Pull {
            socket: PollEvented2::new(self.socket),
            buffer: None,
        }
    }
}

pub struct Pull<P: Poller> {
    socket: P,
    buffer: Option<zmq::Message>,
}

impl<P: Poller> Stream for Pull<P> {
    type Item = zmq::Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Poll Hit!");

        let mut buffer = self.buffer.take().unwrap_or_else(|| zmq::Message::new());

        match self.socket.recv_message(&mut buffer)? {
            Async::Ready(()) => return Ok(Async::Ready(Some(buffer))),
            Async::NotReady => {
                self.buffer = Some(buffer);
                return Ok(Async::NotReady);
            }
        }
    }
}
