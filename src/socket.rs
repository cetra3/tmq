use mio::unix::EventedFd;
use mio::{Evented, Poll, PollOpt, Ready, Token};
use zmq::Socket;

use std::io;

//Wrapper type to allow us to impl Evented & PollEvented
pub struct MioSocket {
    pub io: Socket,
}

impl MioSocket {
    pub fn new(socket: Socket) -> Self {
        Self { io: socket }
    }
}

//Convert to/from MioSocket
impl From<Socket> for MioSocket {
    fn from(socket: Socket) -> Self {
        Self { io: socket }
    }
}

//Convert to/from Socket
impl From<MioSocket> for Socket {
    fn from(socket: MioSocket) -> Self {
        socket.io
    }
}

impl Evented for MioSocket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.io.get_fd()?).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.io.get_fd()?).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.io.get_fd()?).deregister(poll)
    }
}
