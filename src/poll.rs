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
    buffer: Multipart
}

impl EventedSocket {
    /// Creates a new `EventedSocket` from a ZeroMQ socket.
    pub(crate) fn from_zmq_socket(socket: zmq::Socket) -> Self {
        Self {
            socket: PollEvented::new(SocketWrapper::new(socket)),
            buffer: Multipart::new()
        }
    }

    /// Attempt to receive a Multipart message from a ZeroMQ socket.
    ///
    /// Either the whole multipart at once or nothing is received (according to
    /// [http://zguide.zeromq.org/page:all#Multipart-Messages], when one part of a multipart message
    /// has been received, all the others are already available as well).
    ///
    /// If nothing was received, the read flag is cleared.
    pub(crate) fn multipart_recv(&mut self, cx: &mut Context) -> Poll<Option<Result<Multipart>>> {
        let ready = Ready::readable();
        ready!(self.socket.poll_read_ready(cx, ready))?;

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
                    assert!(buffer.is_empty());
                    self.socket.clear_read_ready(cx, ready)?;
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Some(Err(e.into()))),
            }
        }

        assert!(!buffer.is_empty());
        return Poll::Ready(Some(Ok(buffer)));
    }

    /// Register a write notification on this socket.
    /// The socket will be polled once a write can be performed.
    pub(crate) fn multipart_poll_write_flag(&mut self, cx: &mut Context) -> Poll<Result<()>> {
        ready!(self.socket.poll_write_ready(cx))?;
        Poll::Ready(Ok(()))
    }

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
    pub(crate) fn multipart_send(
        &mut self,
        cx: &mut Context,
    ) -> Result<()> {
        let len = self.buffer.len();

        while let Some(msg) = self.buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !self.buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.get_socket().send(&*msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => {
                    self.buffer.push_front(msg);

                    if self.buffer.len() == len {
                        // nothing was written
                        self.socket.clear_write_ready(cx)?;
                    }
                    // The events of the socket must be picked up after EAGAIN
                    self.get_socket().get_events()?;
                    return Ok(());
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    /// Flush the message buffer if there are any messages.
    /// The remaining part of the buffer that was not flushed will be returned, along with the Poll
    /// status.
    ///
    /// If `poll` is true, this function will also register a write notification by calling
    /// `multipart_poll_write_flag`.
    pub(crate) fn multipart_flush(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<()>> {
        // If we have some data in the buffer, attempt to send them.
        ready!(self.multipart_poll_write_flag(cx))?;

        if !self.buffer.is_empty() {
            self.multipart_send(cx)?;
            if !self.buffer.is_empty() {
                return Poll::Pending;
            }
        }

        Poll::Ready(Ok(()))
    }

    pub(crate) fn get_socket(&self) -> &zmq::Socket {
        &self.socket.get_ref().socket
    }
}
