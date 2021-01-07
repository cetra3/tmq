use futures::ready;

use crate::error::TmqError::InterruptedSend;
use crate::socket::{AsZmqSocket, SocketWrapper};
use crate::{Multipart, Result};
use std::{
    collections::VecDeque,
    task::{Context, Poll},
};
use tokio::io::unix::AsyncFd;
use zmq::Socket;

/// Implements functions for asynchronous reading and writing of multipart messages.
pub(crate) struct ZmqPoller(AsyncFd<SocketWrapper>);

impl ZmqPoller {
    #[inline]
    pub(crate) fn from_zmq_socket(socket: zmq::Socket) -> Result<Self> {
        Ok(Self(AsyncFd::new(SocketWrapper::new(socket)?)?))
    }
}

impl AsZmqSocket for ZmqPoller {
    #[inline]
    fn get_socket(&self) -> &Socket {
        &self.0.get_ref().socket
    }
}

impl ZmqPoller {
    /// Attempt to receive a Multipart message from a ZeroMQ socket with buffering.
    ///
    /// If there is any message in the buffer, it will be returned right away.
    /// If not, a batch of messages up to the capacity of the buffer will be read from the socket.
    ///
    /// If nothing was received, the read flag is cleared.
    pub(crate) fn multipart_recv_buffered(
        &self,
        cx: &mut Context<'_>,
        read_buffer: &mut ReceiverBuffer,
    ) -> Poll<Result<Multipart>> {
        if read_buffer.is_empty() {
            ready!(self.multipart_poll_read_ready(cx))?;

            let mut buffer = Multipart::default();
            loop {
                let mut msg = zmq::Message::new();
                match self.get_socket().recv(&mut msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        let more = msg.get_more();
                        buffer.push_back(msg);
                        if !more {
                            read_buffer.push_back(buffer);
                            if read_buffer.is_full() {
                                break Poll::Ready(Ok(read_buffer.pop_front().unwrap()));
                            }

                            buffer = Multipart::default();
                        }
                    }
                    Err(zmq::Error::EAGAIN) => {
                        if !buffer.is_empty() {
                            read_buffer.push_back(buffer);
                        }
                        self.clear_read_ready(cx)?;

                        if read_buffer.is_empty() {
                            break Poll::Pending;
                        } else {
                            break Poll::Ready(Ok(read_buffer.pop_front().unwrap()));
                        }
                    }
                    Err(e) => break Poll::Ready(Err(e.into())),
                }
            }
        } else {
            Poll::Ready(Ok(read_buffer.pop_front().unwrap()))
        }
    }

    /// Attempt to receive a Multipart message from a ZeroMQ socket.
    ///
    /// Either the whole multipart at once or nothing is received (according to
    /// [http://zguide.zeromq.org/page:all#Multipart-Messages], when one part of a multipart message
    /// has been received, all the others are already available as well).
    ///
    /// If nothing was received, the read flag is cleared.
    pub(crate) fn multipart_recv(&self, cx: &mut Context<'_>) -> Poll<Result<Multipart>> {
        ready!(self.multipart_poll_read_ready(cx))?;

        let mut buffer = Multipart::default();
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
                    log::warn!("EAGAIN during first message read");
                    self.clear_read_ready(cx)?;
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        assert!(!buffer.is_empty());
        Poll::Ready(Ok(buffer))
    }

    /// Attempt to send a multipart message.
    ///
    /// Sending the whole message at once may not be possible.
    /// If the function returns `Poll::Ready(Ok(()))`, the whole message has been sent.
    /// If the function returns `Poll::Pending`, the remaining message in the buffer should be
    /// attempted to be written the next time the socket is polled.
    pub(crate) fn multipart_send(&self, buffer: &mut Multipart) -> Poll<Result<()>> {
        let len = buffer.len();

        while let Some(msg) = buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.get_socket().send(&*msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => {
                    buffer.push_front(msg);

                    // If this was not the first message, we return a special error.
                    if buffer.len() != len {
                        return Poll::Ready(Err(InterruptedSend));
                    }
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    /// Attempt to flush the message buffer.
    /// If the buffer cannot be fully flushed, `Poll::Pending` will be returned and a wakeup
    /// will be scheduled the next time there is an event on the ZMQ socket.
    pub(crate) fn multipart_flush(
        &self,
        cx: &mut Context<'_>,
        buffer: &mut Multipart,
    ) -> Poll<Result<()>> {
        while !buffer.is_empty() {
            ready!(self.multipart_poll_write_ready(cx))?;
            ready!(self.multipart_send(buffer))?;
        }

        assert!(buffer.is_empty());
        Poll::Ready(Ok(()))
    }

    /// Returns `Poll::Ready(Ok(()))` if the given ZMQ socket is ready for writing.
    /// Returns `Poll::Pending` and schedules a wakeup on the next event for the socket otherwise.
    pub(crate) fn multipart_poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.multipart_poll(cx, zmq::POLLOUT)
    }

    /// Returns `Poll::Ready(Ok(()))` if the given ZMQ socket is ready for reading.
    /// Returns `Poll::Pending` and schedules a wakeup on the next event for the socket otherwise.
    pub(crate) fn multipart_poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.multipart_poll(cx, zmq::POLLIN)
    }

    fn multipart_poll(&self, cx: &mut Context<'_>, event: zmq::PollEvents) -> Poll<Result<()>> {
        let events = self.get_socket().get_events()?;
        if events.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            self.clear_read_ready(cx)?;
            Poll::Pending
        }
    }

    fn clear_read_ready(&self, cx: &mut Context<'_>) -> Result<()> {
        if let Poll::Ready(mut guard) = self.0.poll_read_ready(cx)? {
            guard.clear_ready();
            cx.waker().wake_by_ref();
        }
        Ok(())
    }
}

/// Buffer used by receiver implementations to hold multiparts.
pub(crate) struct ReceiverBuffer {
    capacity: usize,
    buffer: VecDeque<Multipart>,
}

impl ReceiverBuffer {
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        let buffer = VecDeque::with_capacity(capacity);
        Self { capacity, buffer }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.buffer.len() == self.capacity
    }

    #[inline]
    pub(crate) fn pop_front(&mut self) -> Option<Multipart> {
        self.buffer.pop_front()
    }

    #[inline]
    pub(crate) fn push_back(&mut self, item: Multipart) {
        self.buffer.push_back(item)
    }
}
