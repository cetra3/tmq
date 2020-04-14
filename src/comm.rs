use crate::{
    poll::{ReceiverBuffer, ZmqPoller},
    Multipart,
};

type Poller = ZmqPoller;

/// Sends multiparts using an owned Poller.
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

/// Receives multiparts using an owned Poller.
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

/// Receives buffered multiparts using an owned Poller.
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

/// Sends ands receives multiparts using an owned Poller.
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
}
