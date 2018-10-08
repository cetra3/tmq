use futures::Stream;
use futures::{Async, Poll};

use tokio::reactor::PollEvented2;

use failure::{err_msg, Error};
use std::{fmt, mem};

use mio::Ready;
use futures::task;

use zmq::{self, Context, SocketType};

use mio_socket::MioSocket;

pub fn req(context: &Context) -> ReqBuilder {
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
}

impl ReqBuilderBounded {
    pub fn with<M: Into<zmq::Message>, S: Stream<Item = M>>(self, stream: S) -> Req<M, S> {
        Req {
            stream,
            socket: PollEvented2::new(self.socket),
            state: State::BeginSend,
        }
    }
}

pub struct Req<M: Into<zmq::Message>, S: Stream<Item = M>> {
    stream: S,
    socket: PollEvented2<MioSocket>,
    state: State,
}

impl<M: Into<zmq::Message>, S: Stream<Item = M>> Req<M, S> {
    fn send_message(&mut self, msg: zmq::Message) -> Poll<Option<zmq::Message>, Error> {
        match self.socket.poll_write_ready()? {
            Async::Ready(_) => {

                //Send the message, and if it will block, then we set up a notifier
                if let Err(e) = self.socket.get_ref().io.send(&*msg, zmq::DONTWAIT) {
                    if e == zmq::Error::EAGAIN {
                        self.socket.clear_write_ready()?;
                        self.state = State::Sending(msg);
                        return Ok(Async::NotReady);
                    } else {
                        return Err(e.into());
                    }
                }

                task::current().notify();

                self.state = State::Receiving(zmq::Message::new());
                return Ok(Async::NotReady);
            }

            //If it's not ready to send yet, then we basically need to hold onto the message and wait
            Async::NotReady => {
                self.state = State::Sending(msg);
                return Ok(Async::NotReady);
            }
        }
    }

    fn recv_message(&mut self, mut msg: zmq::Message) -> Poll<Option<zmq::Message>, Error> {
        let ready = Ready::readable();

        match self.socket.poll_read_ready(ready)? {
            Async::Ready(_) => {
                if let Err(e) = self.socket.get_ref().io.recv(&mut msg, zmq::DONTWAIT) {
                    if e == zmq::Error::EAGAIN {
                        self.socket.clear_read_ready(ready)?;
                        self.state = State::Receiving(msg);
                        return Ok(Async::NotReady);
                    } else {
                        return Err(e.into());
                    }
                }

                task::current().notify();

                self.state = State::BeginSend;
                return Ok(Async::Ready(Some(msg)));
            }
            Async::NotReady => {
                self.state = State::Receiving(msg);

                return Ok(Async::NotReady);
            }
        }
    }
}

pub enum State {
    BeginSend,
    Sending(zmq::Message),
    Receiving(zmq::Message),
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

impl<M: Into<zmq::Message>, S: Stream<Item = M>> Stream for Req<M, S> {
    type Item = zmq::Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("Req poll {:?}", self.state);

        let state = mem::replace(&mut self.state, State::InPoll);

        match state {
            State::BeginSend => {
                match self
                    .stream
                    .poll()
                    .map_err(|_| err_msg("Stream Poll Error"))?
                {
                    //The inner stream is not ready to send anything yet, so we wait.
                    Async::NotReady => {
                        self.state = State::BeginSend;
                        return Ok(Async::NotReady);
                    }
                    //The inner stream is ready to send, check the zmq socket readiness
                    Async::Ready(Some(msg)) => {
                        return self.send_message(msg.into());
                    }
                    //The inner stream is finished, so are we
                    Async::Ready(None) => {
                        return Ok(Async::Ready(None));
                    }
                }
            }
            State::Sending(msg) => {
                return self.send_message(msg);
            }
            State::Receiving(msg) => {
                return self.recv_message(msg);
            }
            State::InPoll => unreachable!("Should not get here"),
        }
    }
}
