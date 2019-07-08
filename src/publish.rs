use crate::TmqMessage;
use futures::{task, Async, AsyncSink, Poll, Sink, StartSend};

use tokio::reactor::PollEvented2;

use failure::Error;

use std::collections::VecDeque;

use zmq::{self, Context, SocketType};

use crate::poll::Poller;
use crate::socket::MioSocket;

pub fn publish(context: &Context) -> PubBuilder {
    PubBuilder { context }
}

pub struct PubBuilder<'a> {
    context: &'a Context,
}

impl<'a> PubBuilder<'a> {
    pub fn bind<M: Into<TmqMessage>>(
        self,
        endpoint: &str,
    ) -> Result<Pub<M, PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PUB)?;
        socket.bind(endpoint)?;

        Ok(Pub {
            socket: PollEvented2::new(socket.into()),
            buffer: VecDeque::new(),
            current: None,
        })
    }

    pub fn connect<M: Into<TmqMessage>>(
        self,
        endpoint: &str,
    ) -> Result<Pub<M, PollEvented2<MioSocket>>, Error> {
        let socket = self.context.socket(SocketType::PUB)?;
        socket.connect(endpoint)?;

        Ok(Pub {
            socket: PollEvented2::new(socket.into()),
            buffer: VecDeque::new(),
            current: None,
        })
    }
}

pub struct Pub<M: Into<TmqMessage>, P: Poller> {
    pub(crate) socket: P,
    pub(crate) buffer: VecDeque<M>,
    pub(crate) current: Option<TmqMessage>,
}

impl<P: Poller, M: Into<TmqMessage>> Sink for Pub<M, P> {
    type SinkItem = M;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if self.current.is_none() {
            self.current = Some(item.into());
        } else {
            self.buffer.push_back(item);
        }

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        debug!("Poll complete hit!");

        if let Some(msg) = self.current.take() {
            match self.socket.send_message(&msg)? {
                Async::NotReady => {
                    //Plop it back into our queue
                    self.current = Some(msg);
                    return Ok(Async::NotReady);
                }
                Async::Ready(()) => {
                    if let Some(new_msg) = self.buffer.pop_front().map(|val| val.into()) {
                        //Message was sent, add a notify to be polled once more to check whether there are any messages.
                        task::current().notify();

                        self.current = Some(new_msg);
                        return Ok(Async::NotReady);
                    }
                }
            }
        }

        return Ok(Async::Ready(()));
    }
}
