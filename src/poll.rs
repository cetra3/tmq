use std::task::Context;

use futures::{Poll, ready};
use mio::Ready;
use tokio::reactor::PollEvented;
use zmq;

use crate::{Multipart, Result};
use crate::socket::SocketWrapper;

pub(crate) struct EventedSocket(pub(crate) PollEvented<SocketWrapper>);

// TODO: define two streams - one for reading multipart messages, another for reading single
// messages directly
impl EventedSocket {
    /// http://zguide.zeromq.org/page:all#Multipart-Messages
    /// When one part of a multipart message has been received, all the others are received as well.
    /// There should thus be no need to keep a buffer outside this method.
    pub(crate) fn poll_receive_multipart(&mut self, cx: &mut Context) -> Poll<Option<Result<Multipart>>> {
        let ready = Ready::readable();
        ready!(self.0.poll_read_ready(cx, ready))?;

        let mut buffer = Multipart::new();
        loop {
            let mut msg = zmq::Message::new();
            match self.0.get_ref().socket.recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    let more = msg.get_more();
                    buffer.push(msg);
                    if !more
                    {
                        break;
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    assert!(buffer.is_empty());
                    self.0.clear_read_ready(cx, ready)?;
                    return Poll::Pending;
                },
                Err(e) => return Poll::Ready(Some(Err(e.into())))
            }
        }

        assert!(!buffer.is_empty());
        return Poll::Ready(Some(Ok(buffer)));
    }
}
