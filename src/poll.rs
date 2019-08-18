use std::task::Context;

use futures::{ready, Poll};
use mio::Ready;
use tokio::reactor::PollEvented;
use zmq;

use crate::socket::SocketWrapper;
use crate::{Multipart, Result};

/// Wrapper on top of a ZeroMQ socket, implements functions for asynchronous reading and writing
/// of multipart messages.
pub(crate) struct EventedSocket(pub(crate) PollEvented<SocketWrapper>);

impl EventedSocket {
    /// Creates a new `EventedSocket` from a ZeroMQ socket.
    pub(crate) fn from_zmq_socket(socket: zmq::Socket) -> Self {
        Self(PollEvented::new(SocketWrapper::new(socket)))
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
        ready!(self.0.poll_read_ready(cx, ready))?;

        let mut buffer = Multipart::new();
        loop {
            let mut msg = zmq::Message::new();
            match self.0.get_ref().socket.recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    let more = msg.get_more();
                    buffer.push_back(msg);
                    if !more {
                        break;
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    assert!(buffer.is_empty());
                    self.0.clear_read_ready(cx, ready)?;
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
        ready!(self.0.poll_write_ready(cx))?;
        Poll::Ready(Ok(()))
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
        mut item: Multipart,
    ) -> Result<Option<Multipart>> {
        let len = item.len();
        while let Some(msg) = item.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !item.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.0.get_ref().socket.send(&*msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => {
                    item.push_front(msg);

                    if item.len() == len {
                        // nothing was written
                        self.0.clear_write_ready(cx)?;
                    }
                    return Ok(Some(item));
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(None)
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
        buffer: Option<Multipart>,
        poll: bool,
    ) -> (Poll<Result<()>>, Option<Multipart>) {
        // If we have some data in the buffer, attempt to send them.
        if let Some(data) = buffer {
            match self.multipart_send(cx, data) {
                Ok(remaining) => {
                    // Some data is still remaining, return a pending status and the remaining data.
                    match remaining {
                        Some(_) => return (Poll::Pending, remaining),
                        None => {}
                    }
                }
                Err(e) => return (Poll::Ready(Err(e)), None),
            }
        }

        // register write event notification
        if poll {
            (self.multipart_poll_write_flag(cx), None)
        } else {
            (Poll::Ready(Ok(())), None)
        }
    }
}
