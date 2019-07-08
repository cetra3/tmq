use crate::TmqMessage;
use futures::Stream;
use futures::{Async, Poll};

use tokio::reactor::PollEvented2;

use failure::Error;
use std::{fmt, mem};

use futures::task;

use zmq::{self, Context, SocketType};

use crate::poll::Poller;
use crate::socket::MioSocket;

pub fn request(context: &Context) -> ReqBuilder {
    ReqBuilder { context }
}

pub struct ReqBuilder<'a> {
    context: &'a Context,
}

pub struct ReqBuilderBounded {
    socket: MioSocket,
}

impl<'a> ReqBuilder<'a> {
    pub fn connect(self, endpoint: &str) -> Result<ReqBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::REQ)?;
        socket.connect(endpoint)?;

        Ok(ReqBuilderBounded {
            socket: socket.into(),
        })
    }

    pub fn bind(self, endpoint: &str) -> Result<ReqBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::REQ)?;
        socket.bind(endpoint)?;

        Ok(ReqBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl ReqBuilderBounded {
    pub fn with<M: Into<TmqMessage>, S: Stream<Item = M, Error = Error>>(
        self,
        stream: S,
    ) -> Req<M, S, PollEvented2<MioSocket>> {
        Req {
            stream,
            socket: PollEvented2::new(self.socket),
            state: State::BeginSend,
        }
    }
}

pub struct Req<M: Into<TmqMessage>, S: Stream<Item = M>, P: Poller> {
    stream: S,
    socket: P,
    state: State,
}

pub enum State {
    BeginSend,
    Sending(TmqMessage),
    Receiving(TmqMessage),
    InPoll,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::BeginSend => write!(f, "State:BeginSend"),
            State::Sending(_) => write!(f, "State:Sending(<Message>)"),
            State::Receiving(_) => write!(f, "State:Receiving(<Message>)"),
            State::InPoll => write!(f, "State:InPoll"),
        }
    }
}

impl<M: Into<TmqMessage>, S: Stream<Item = M, Error = Error>, P: Poller> Stream for Req<M, S, P> {
    type Item = zmq::Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Req poll {:?}", self.state);

        let state = mem::replace(&mut self.state, State::InPoll);

        match state {
            State::BeginSend => {
                match self.stream.poll()? {
                    //The inner stream is not ready to send anything yet, so we wait.
                    Async::NotReady => {
                        self.state = State::BeginSend;
                    }
                    //The inner stream is ready to send, set the state to sending
                    Async::Ready(Some(msg)) => {
                        task::current().notify();
                        self.state = State::Sending(msg.into());
                    }
                    //The inner stream is finished, so are we
                    Async::Ready(None) => {
                        return Ok(Async::Ready(None));
                    }
                }
            }
            State::Sending(msg) => match self.socket.send_message(&msg)? {
                Async::Ready(_) => {
                    task::current().notify();
                    self.state = State::Receiving(TmqMessage::Single(zmq::Message::new()));
                }
                Async::NotReady => {
                    self.state = State::Sending(msg);
                }
            },
            State::Receiving(mut msg) => match self.socket.recv_message(&mut msg)? {
                Async::Ready(_) => {
                    task::current().notify();

                    self.state = State::BeginSend;

                    match msg {
                        TmqMessage::Multipart(mut msgs) => return Ok(Async::Ready(msgs.pop())),
                        TmqMessage::Single(msg) => return Ok(Async::Ready(Some(msg))),
                    }
                }
                Async::NotReady => {
                    self.state = State::Receiving(msg);
                }
            },
            State::InPoll => unreachable!("Should not get here"),
        };

        return Ok(Async::NotReady);
    }
}
