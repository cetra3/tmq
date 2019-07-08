use crate::TmqMessage;
use futures::{Async, Poll, Stream};

use tokio::reactor::PollEvented2;

use failure::Error;

use zmq::{self, Context, SocketType};

use crate::poll::Poller;
use crate::socket::MioSocket;

pub fn subscribe(context: &Context) -> SubBuilder {
    SubBuilder { context }
}

pub struct SubBuilder<'a> {
    context: &'a Context,
}

pub struct SubBuilderBounded {
    socket: MioSocket,
}

impl<'a> SubBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<SubBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::SUB)?;
        socket.connect(endpoint)?;

        Ok(SubBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<SubBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::SUB)?;
        socket.bind(endpoint)?;

        Ok(SubBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl SubBuilderBounded {
    pub fn subscribe<'a, R: AsRef<[u8]>>(self, topic: R) -> Sub<PollEvented2<MioSocket>> {
        //Will only fail for non-rusty reasons: http://api.zeromq.org/2-1:zmq-setsockopt#toc20
        self.socket
            .io
            .set_subscribe(topic.as_ref())
            .expect("Couldn't set Subscribe");

        Sub {
            socket: PollEvented2::new(self.socket),
            buffer: None,
        }
    }

    pub fn subscribe_mpart<'a, R: AsRef<[u8]>>(
        self,
        topic: R,
    ) -> SubMpart<PollEvented2<MioSocket>> {
        //Will only fail for non-rusty reasons: http://api.zeromq.org/2-1:zmq-setsockopt#toc20
        self.socket
            .io
            .set_subscribe(topic.as_ref())
            .expect("Couldn't set Subscribe");

        SubMpart {
            socket: PollEvented2::new(self.socket),
            buffer: None,
        }
    }
}

pub struct Sub<P: Poller> {
    pub(crate) socket: P,
    pub(crate) buffer: Option<TmqMessage>,
}

impl<P: Poller> Stream for Sub<P> {
    type Item = zmq::Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Poll Hit!");

        let mut buffer = self
            .buffer
            .take()
            .unwrap_or_else(|| TmqMessage::Single(zmq::Message::new()));

        match self.socket.recv_message(&mut buffer)? {
            Async::Ready(()) => match buffer {
                TmqMessage::Multipart(mut msgs) => return Ok(Async::Ready(msgs.pop())),
                TmqMessage::Single(msg) => return Ok(Async::Ready(Some(msg))),
            },
            Async::NotReady => {
                self.buffer = Some(buffer);
                return Ok(Async::NotReady);
            }
        }
    }
}

pub struct SubMpart<P: Poller> {
    pub(crate) socket: P,
    pub(crate) buffer: Option<TmqMessage>,
}

impl<P: Poller> Stream for SubMpart<P> {
    type Item = Vec<zmq::Message>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Poll Hit!");

        let mut buffer = self
            .buffer
            .take()
            .unwrap_or_else(|| TmqMessage::Multipart(Vec::new()));

        match self.socket.recv_message(&mut buffer)? {
            Async::Ready(()) => match buffer {
                TmqMessage::Multipart(msgs) => return Ok(Async::Ready(Some(msgs))),
                TmqMessage::Single(msg) => return Ok(Async::Ready(Some(vec![msg]))),
            },
            Async::NotReady => {
                self.buffer = Some(buffer);
                return Ok(Async::NotReady);
            }
        }
    }
}
