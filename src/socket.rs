use std::io;

use mio::unix::EventedFd;
use mio::{Evented, Poll, PollOpt, Ready, Token};

pub(crate) struct SocketWrapper {
    pub(crate) socket: zmq::Socket,
}

impl SocketWrapper {
    pub fn new(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl Evented for SocketWrapper {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).register(poll, token, interest, opts)
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).reregister(poll, token, interest, opts)
    }
    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).deregister(poll)
    }
}
