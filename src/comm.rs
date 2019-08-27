use crate::poll::{ReceiverBuffer, ZmqPoller};
use crate::Multipart;
use std::rc::Rc;

type Poller = ZmqPoller;
type PollerShared = Rc<Poller>;

pub struct Sender {
    pub(crate) poller: Poller,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(Sender, poller);
impl_sink!(Sender, buffer, poller);

impl Sender {
    pub(crate) fn new(poller: Poller) -> Self {
        Self {
            poller,
            buffer: Default::default(),
        }
    }
}

pub struct Receiver {
    pub(crate) poller: Poller,
}
impl_as_socket!(Receiver, poller);
impl_stream!(Receiver, poller);

impl Receiver {
    pub(crate) fn new(poller: Poller) -> Self {
        Self { poller }
    }
    pub fn buffered(self, capacity: usize) -> BufferedReceiver {
        BufferedReceiver::new(self.poller, capacity)
    }
}

pub struct BufferedReceiver {
    pub(crate) poller: Poller,
    pub(crate) buffer: ReceiverBuffer,
}
impl_as_socket!(BufferedReceiver, poller);
impl_buffered_stream!(BufferedReceiver, buffer, poller);

impl BufferedReceiver {
    pub(crate) fn new(poller: Poller, capacity: usize) -> Self {
        Self {
            poller,
            buffer: ReceiverBuffer::new(capacity),
        }
    }
}

pub struct SenderReceiver {
    pub(crate) poller: Poller,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(SenderReceiver, poller);
impl_sink!(SenderReceiver, buffer, poller);
impl_stream!(SenderReceiver, poller);

impl SenderReceiver {
    pub(crate) fn new(poller: Poller) -> Self {
        Self {
            poller,
            buffer: Default::default(),
        }
    }
    pub fn split(self) -> (SharedReceiver, SharedSender) {
        let rc = Rc::new(self.poller);
        (
            SharedReceiver::new(rc.clone()),
            SharedSender::new(rc, self.buffer),
        )
    }
}

/// Wrappers that share a socket
pub struct SharedSender {
    pub(crate) poller: PollerShared,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(SharedSender, poller);
impl_sink!(SharedSender, buffer, poller);

impl SharedSender {
    pub(crate) fn new(poller: PollerShared, buffer: Multipart) -> Self {
        Self { poller, buffer }
    }
}

pub struct SharedReceiver {
    pub(crate) poller: PollerShared,
}
impl_as_socket!(SharedReceiver, poller);
impl_stream!(SharedReceiver, poller);

impl SharedReceiver {
    pub(crate) fn new(poller: PollerShared) -> Self {
        Self { poller }
    }
    pub fn buffered(self, capacity: usize) -> SharedBufferedReceiver {
        SharedBufferedReceiver::new(self.poller, capacity)
    }
}

pub struct SharedBufferedReceiver {
    pub(crate) poller: PollerShared,
    pub(crate) buffer: ReceiverBuffer,
}
impl_as_socket!(SharedBufferedReceiver, poller);
impl_buffered_stream!(SharedBufferedReceiver, buffer, poller);

impl SharedBufferedReceiver {
    pub(crate) fn new(poller: PollerShared, capacity: usize) -> Self {
        Self {
            poller,
            buffer: ReceiverBuffer::new(capacity),
        }
    }
}
