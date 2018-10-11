use failure::Error;
use futures::{Async, Poll};
use futures::{Future, IntoFuture};
use socket::MioSocket;

use poll::Poller;
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

pub fn respond(context: &Context) -> RepBuilder {
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

    pub fn connect(self, endpoint: &str) -> Result<RepBuilderBounded, Error> {
        let socket = self.context.socket(SocketType::REP)?;
        socket.connect(endpoint)?;

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
            responder,
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

impl<R: Responder<Output = F>, F: Future<Item = zmq::Message, Error = Error>> Future for Rep<R, F> {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("Rep poll {:?}", self.state);

        let state = mem::replace(&mut self.state, State::InPoll);

        match state {
            State::Receiving(mut msg) => match self.socket.recv_message(&mut msg)? {
                Async::Ready(_) => {
                    task::current().notify();
                    self.state = State::RunningFuture(self.responder.respond(msg));
                }
                Async::NotReady => {
                    self.state = State::Receiving(msg);
                }
            },
            State::RunningFuture(mut f) => match f.poll()? {
                Async::Ready(msg) => {
                    task::current().notify();
                    self.state = State::Sending(msg);
                }
                Async::NotReady => {
                    self.state = State::RunningFuture(f);
                }
            },
            State::Sending(msg) => match self.socket.send_message(&msg)? {
                Async::Ready(_) => {
                    task::current().notify();
                    self.state = State::Receiving(zmq::Message::new());
                }
                Async::NotReady => {
                    self.state = State::Sending(msg);
                }
            },
            State::InPoll => unreachable!("Should not get here"),
        };

        return Ok(Async::NotReady);
    }
}
