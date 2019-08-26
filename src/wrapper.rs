use crate::poll::{ReceiveBuffer, ZmqPoller};
use crate::Multipart;
use std::rc::Rc;

type Poller = ZmqPoller;
type PollerShared = Rc<Poller>;

pub struct SendWrapper {
    pub(crate) poller: Poller,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(SendWrapper, poller);
impl_sink!(SendWrapper, buffer, poller);

impl SendWrapper {
    pub(crate) fn new(poller: Poller) -> Self {
        Self {
            poller,
            buffer: Default::default(),
        }
    }
}

pub struct ReceiveWrapper {
    pub(crate) poller: Poller,
}
impl_as_socket!(ReceiveWrapper, poller);
impl_stream!(ReceiveWrapper, poller);

impl ReceiveWrapper {
    pub(crate) fn new(poller: Poller) -> Self {
        Self { poller }
    }
    pub fn buffered(self, capacity: usize) -> BufferedReceiveWrapper {
        BufferedReceiveWrapper::new(self.poller, capacity)
    }
}

pub struct BufferedReceiveWrapper {
    pub(crate) poller: Poller,
    pub(crate) buffer: ReceiveBuffer,
}
impl_as_socket!(BufferedReceiveWrapper, poller);
impl_buffered_stream!(BufferedReceiveWrapper, buffer, poller);

impl BufferedReceiveWrapper {
    pub(crate) fn new(poller: Poller, capacity: usize) -> Self {
        Self {
            poller,
            buffer: ReceiveBuffer::new(capacity),
        }
    }
}

pub struct SendReceiveWrapper {
    pub(crate) poller: Poller,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(SendReceiveWrapper, poller);
impl_sink!(SendReceiveWrapper, buffer, poller);
impl_stream!(SendReceiveWrapper, poller);

impl SendReceiveWrapper {
    pub(crate) fn new(poller: Poller) -> Self {
        Self {
            poller,
            buffer: Default::default(),
        }
    }
    pub fn split(self) -> (ReceiveWrapperShared, SendWrapperShared) {
        let rc = Rc::new(self.poller);
        (
            ReceiveWrapperShared::new(rc.clone()),
            SendWrapperShared::new(rc, self.buffer),
        )
    }
}

/// Wrappers that share a socket
pub struct SendWrapperShared {
    pub(crate) poller: PollerShared,
    pub(crate) buffer: Multipart,
}
impl_as_socket!(SendWrapperShared, poller);
impl_sink!(SendWrapperShared, buffer, poller);

impl SendWrapperShared {
    pub(crate) fn new(poller: PollerShared, buffer: Multipart) -> Self {
        Self { poller, buffer }
    }
}

pub struct ReceiveWrapperShared {
    pub(crate) poller: PollerShared,
}
impl_as_socket!(ReceiveWrapperShared, poller);
impl_stream!(ReceiveWrapperShared, poller);

impl ReceiveWrapperShared {
    pub(crate) fn new(poller: PollerShared) -> Self {
        Self { poller }
    }
    pub fn buffered(self, capacity: usize) -> BufferedReceiveWrapperShared {
        BufferedReceiveWrapperShared::new(self.poller, capacity)
    }
}

pub struct BufferedReceiveWrapperShared {
    pub(crate) poller: PollerShared,
    pub(crate) buffer: ReceiveBuffer,
}
impl_as_socket!(BufferedReceiveWrapperShared, poller);
impl_buffered_stream!(BufferedReceiveWrapperShared, buffer, poller);

impl BufferedReceiveWrapperShared {
    pub(crate) fn new(poller: PollerShared, capacity: usize) -> Self {
        Self {
            poller,
            buffer: ReceiveBuffer::new(capacity),
        }
    }
}
