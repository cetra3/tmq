use failure::Error;
use futures::{Async, Poll};
use futures::{Future, IntoFuture};
use mio_socket::MioSocket;

use mio::Ready;
use tokio::reactor::PollEvented2;

use futures::task;

use std::{fmt, mem};

use zmq::{self, Context, SocketType};

pub trait Responder {
    type Output: Future<Item = zmq::Message, Error = Error>;
    fn respond(&mut self, zmq::Message) -> Self::Output;
}

impl<
        I: IntoFuture<Future = F, Item = zmq::Message, Error = Error>,
        F: Future<Item = zmq::Message, Error = Error>,
        M: FnMut(zmq::Message) -> I,
    > Responder for M
{
    type Output = F;

    fn respond(&mut self, msg: zmq::Message) -> Self::Output {
        self(msg).into_future()
    }
}

pub fn rep(context: &Context) -> RepBuilder {
    RepBuilder { context }
}

pub struct RepBuilder<'a> {
    context: &'a Context,
}

pub struct RepBuilderBounded {
    socket: MioSocket,
}

impl<'a> RepBuilder<'a> {
    pub fn bind(self, endpoint: &str) -> Result<RepBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::REP)?;
        socket.bind(endpoint)?;

        Ok(RepBuilderBounded {
            socket: socket.into(),
        })
    }
}

impl RepBuilderBounded {
    pub fn with<R: Responder, F: Future<Item = zmq::Message, Error = Error>>(
        self,
        responder: R,
    ) -> Rep<R, F> {
        Rep {
            socket: PollEvented2::new(self.socket),
            state: State::Receiving(zmq::Message::new()),
            responder
        }
    }
}

pub struct Rep<R: Responder, F: Future<Item = zmq::Message, Error = Error>> {
    socket: PollEvented2<MioSocket>,
    state: State<F>,
    responder: R,
}


pub enum State<F: Future<Item = zmq::Message, Error = Error>> {
    Receiving(zmq::Message),
    RunningFuture(F),
    Sending(zmq::Message),
    InPoll,
}

impl<F: Future<Item = zmq::Message, Error = Error>> fmt::Debug for State<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::Receiving(_) => write!(f, "State:Receiving(<Message>)"),
            State::RunningFuture(_) => write!(f, "State:RunningFuture(<Future>)"),
            State::Sending(_) => write!(f, "State:Sending(<Message>)"),
            State::InPoll => write!(f, "State:InPoll"),
        }
    }
}

impl<R: Responder<Output = F>, F: Future<Item = zmq::Message, Error = Error>> Rep<R, F> {
    fn send_message(&mut self, msg: zmq::Message) -> Poll<(), Error> {
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

                self.state = State::Receiving(zmq::Message::new());
                task::current().notify();
                return Ok(Async::NotReady);
            }

            //If it's not ready to send yet, then we basically need to hold onto the message and wait
            Async::NotReady => {
                self.state = State::Sending(msg);
                return Ok(Async::NotReady);
            }
        }
    }

    fn recv_message(&mut self, mut msg: zmq::Message) -> Poll<(), Error> {
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

                let mut future = self.responder.respond(msg);
                self.state = State::RunningFuture(future);
                task::current().notify();
                return Ok(Async::NotReady);
            }
            Async::NotReady => {
                self.state = State::Receiving(msg);
                return Ok(Async::NotReady);
            }
        }
    }
}

impl<R: Responder<Output = F>, F: Future<Item = zmq::Message, Error = Error>> Future for Rep<R, F> {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("Rep poll {:?}", self.state);

        let state = mem::replace(&mut self.state, State::InPoll);

        match state {
            State::Receiving(msg) => {
                return self.recv_message(msg);
            }
            State::RunningFuture(mut f) => match f.poll()? {
                Async::Ready(msg) => {
                    return self.send_message(msg);
                }
                Async::NotReady => {
                    self.state = State::RunningFuture(f);
                    task::current().notify();
                    return Ok(Async::NotReady);
                }
            },
            State::Sending(msg) => {
                return self.send_message(msg);
            }
            State::InPoll => unreachable!("Should not get here"),
        }
    }
}
