use crate::socket::MioSocket;
use crate::TmqMessage;
use failure::Error;
use futures::{Async, Poll};
use futures::{Future, IntoFuture};

use crate::poll::Poller;
use tokio::reactor::PollEvented2;

use futures::task;

use std::{fmt, mem};

use zmq::{self, Context, SocketType};

/// Responder trait for `REP` style sockets. 
/// 
/// The Responder can return a future of any type that implements `Into<TmqMessage>`
/// 
/// ```rust
/// # use tmq::*;
/// # use failure::Error;
/// # use futures::future::FutureResult;
/// pub struct EchoResponder {}
/// 
/// impl Responder<TmqMessage> for EchoResponder {
///     type Output = FutureResult<TmqMessage, Error>;
/// 
///     fn respond(&mut self, msg: TmqMessage) -> Self::Output {
///         return FutureResult::from(Ok(msg));
///     }
/// }
/// ```
/// 
/// Alternatively, you can use a free-standing function
/// 
/// ```rust
/// # use tmq::*;
/// # use failure::Error;
/// # use futures::future::ok;
/// # use futures::Future;
/// fn echo(msg: TmqMessage) -> impl Future<Item = TmqMessage, Error = Error> {
///   return ok(msg);
/// }
/// ```
/// 
pub trait Responder<T: Into<TmqMessage>> {
    type Output: Future<Item = T, Error = Error>;
    fn respond(&mut self, msg: T) -> Self::Output;
}

/// This allows functions and closures to implement the `Responder` trait if the message type implements `Into<TmqMessage>` and it returns an `IntoFuture<TmqMessage, Error>`
impl<
        T: Into<TmqMessage>,
        I: IntoFuture<Future = F, Item = T, Error = Error>,
        F: Future<Item = T, Error = Error>,
        M: FnMut(T) -> I,
    > Responder<T> for M
{
    type Output = F;

    fn respond(&mut self, msg: T) -> Self::Output {
        self(msg).into_future()
    }
}

/// Allows the `REP` style socket to be created.  See the `response` example
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
    pub fn with<T: Into<TmqMessage>, R: Responder<T>, F: Future<Item = T, Error = Error>>(
        self,
        responder: R,
    ) -> Rep<T, R, F> {
        Rep {
            socket: PollEvented2::new(self.socket),
            state: State::InPoll,
            responder,
        }
    }
}

pub struct Rep<T: Into<TmqMessage>, R: Responder<T>, F: Future<Item = T, Error = Error>> {
    socket: PollEvented2<MioSocket>,
    state: State<T, F>,
    responder: R,
}

pub enum State<T: Into<TmqMessage>, F: Future<Item = T, Error = Error>> {
    Receiving(TmqMessage),
    RunningFuture(F),
    Sending(TmqMessage),
    InPoll,
}

impl<T: Into<TmqMessage>, F: Future<Item = T, Error = Error>> fmt::Debug for State<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::Receiving(_) => write!(f, "State:Receiving(<Message>)"),
            State::RunningFuture(_) => write!(f, "State:RunningFuture(<Future>)"),
            State::Sending(_) => write!(f, "State:Sending(<Message>)"),
            State::InPoll => write!(f, "State:InPoll"),
        }
    }
}

impl<R: Responder<TmqMessage, Output = F>, F: Future<Item = TmqMessage, Error = Error>> Future
    for Rep<TmqMessage, R, F>
{
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

                    self.state = State::Receiving(msg);
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

impl<R: Responder<zmq::Message, Output = F>, F: Future<Item = zmq::Message, Error = Error>> Future
    for Rep<zmq::Message, R, F>
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("Rep poll {:?}", self.state);

        let state = mem::replace(&mut self.state, State::InPoll);

        match state {
            State::Receiving(mut msg) => match self.socket.recv_message(&mut msg)? {
                Async::Ready(_) => {
                    task::current().notify();

                    self.state = State::RunningFuture(self.responder.respond(msg.into()));
                }
                Async::NotReady => {
                    self.state = State::Receiving(msg);
                }
            },
            State::RunningFuture(mut f) => match f.poll()? {
                Async::Ready(msg) => {
                    task::current().notify();
                    self.state = State::Sending(msg.into());
                }
                Async::NotReady => {
                    self.state = State::RunningFuture(f);
                }
            },
            State::Sending(msg) => match self.socket.send_message(&msg)? {
                Async::Ready(_) => {
                    task::current().notify();
                    self.state = State::Receiving(msg);
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
