use std::task::Context;

use futures::{ready, Poll};
use mio::Ready;
use tokio::reactor::PollEvented;
use zmq;

use crate::socket::SocketWrapper;
use crate::{Multipart, Result};

/// Wrapper on top of a ZeroMQ socket, implements functions for asynchronous reading and writing
/// of multipart messages.
pub(crate) struct EventedSocket {
    socket: PollEvented<SocketWrapper>,
    buffer: Multipart,
}

impl EventedSocket {
    /// Creates a new `EventedSocket` from a ZeroMQ socket.
    pub(crate) fn from_zmq_socket(socket: zmq::Socket) -> Self {
        Self {
            socket: PollEvented::new(SocketWrapper::new(socket)),
            buffer: Multipart::new(),
        }
    }

    /// Attempt to receive a Multipart message from a ZeroMQ socket.
    ///
    /// Either the whole multipart at once or nothing is received (according to
    /// [http://zguide.zeromq.org/page:all#Multipart-Messages], when one part of a multipart message
    /// has been received, all the others are already available as well).
    ///
    /// If nothing was received, the read flag is cleared.
    pub(crate) fn multipart_recv(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Multipart>>> {
        let events = self.get_socket().get_events()?;
        if !events.contains(zmq::POLLIN) {
            self.socket.clear_read_ready(cx, Ready::readable())?;
            return Poll::Pending;
        }

        let mut buffer = Multipart::new();
        loop {
            let mut msg = zmq::Message::new();
            match self.get_socket().recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    let more = msg.get_more();
                    buffer.push_back(msg);
                    if !more {
                        break;
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    assert!(false);
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Some(Err(e.into()))),
            }
        }

        assert!(!buffer.is_empty());
        return Poll::Ready(Some(Ok(buffer)));
    }

    /// Store a multipart that should be sent during the next flush.
    pub(crate) fn multipart_set_buffer(&mut self, buffer: Multipart) {
        assert!(self.buffer.is_empty());
        self.buffer = buffer;
    }

    /// Attempt to send a multipart message.
    ///
    /// Sending the whole message at once may not be possible.
    /// If the function returns None, the whole message has been sent.
    /// If the function returns Some(buffer), the remaining messages in the buffer should be
    /// attempted to be written the next time the socket is polled.
    ///
    /// If no message was written, the write flag is cleared.
    pub(crate) fn multipart_send(&mut self) -> Poll<Result<()>> {
        while let Some(msg) = self.buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !self.buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.get_socket().send(&*msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => {
                    self.buffer.push_front(msg);
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    /// Flush the message buffer if there are any messages.
    /// The remaining part of the buffer that was not flushed will be returned, along with the Poll
    /// status.
    ///
    /// If `poll` is true, this function will also register a write notification by calling
    /// `multipart_poll_write_flag`.
    pub(crate) fn multipart_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        while !self.buffer.is_empty() {
            ready!(self.multipart_poll_ready(cx))?;
            ready!(self.multipart_send())?;
        }

        assert!(self.buffer.is_empty());
        Poll::Ready(Ok(()))
    }

    pub(crate) fn multipart_poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let events = self.get_socket().get_events()?;
        if events.contains(zmq::POLLOUT) {
            Poll::Ready(Ok(()))
        } else {
            self.socket.clear_read_ready(cx, Ready::readable())?;
            return Poll::Ready(Ok(()));
        }
    }

    pub(crate) fn get_socket(&self) -> &zmq::Socket {
        &self.socket.get_ref().socket
    }
}
