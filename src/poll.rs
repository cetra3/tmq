use crate::socket::MioSocket;
use crate::TmqMessage;
use failure::Error;
use futures::{Async, Poll};

use mio::Ready;
use tokio::reactor::PollEvented2;

use zmq;

pub trait Poller {
    fn send_message(&self, msg: &TmqMessage) -> Poll<(), Error>;

    fn recv_message(&self, msg: &mut TmqMessage) -> Poll<(), Error>;
}

impl Poller for PollEvented2<MioSocket> {
    fn send_message(&self, tmq_msg: &TmqMessage) -> Poll<(), Error> {
        match self.poll_write_ready()? {
            Async::Ready(_) => {
                let len = (*tmq_msg).len();

                for (i, msg) in (*tmq_msg).iter().enumerate() {
                    let flags = if i < len - 1 {
                        zmq::DONTWAIT | zmq::SNDMORE
                    } else {
                        zmq::DONTWAIT
                    };

                    //Send the message, and if it will block, then we set up a notifier
                    if let Err(e) = self.get_ref().io.send(&**msg, flags) {
                        if e == zmq::Error::EAGAIN {
                            self.clear_write_ready()?;
                            return Ok(Async::NotReady);
                        } else {
                            return Err(e.into());
                        }
                    }
                }

                return Ok(Async::Ready(()));
            }

            //If it's not ready to send yet, then we basically need to hold onto the message and wait
            Async::NotReady => {
                return Ok(Async::NotReady);
            }
        }
    }

    fn recv_message(&self, tmq_msg: &mut TmqMessage) -> Poll<(), Error> {
        let ready = Ready::readable();

        match self.poll_read_ready(ready)? {
            Async::Ready(_) => {
                loop {
                    let mut msg = (*tmq_msg).pop().unwrap_or_else(|| zmq::Message::new());

                    if let Err(e) = self.get_ref().io.recv(&mut msg, zmq::DONTWAIT) {
                        if e == zmq::Error::EAGAIN {
                            self.clear_read_ready(ready)?;
                            (*tmq_msg).push(msg);
                            return Ok(Async::NotReady);
                        } else {
                            return Err(e.into());
                        }
                    }

                    (*tmq_msg).push(msg);

                    if !self.get_ref().io.get_rcvmore()? {
                        break;
                    } else {
                        (*tmq_msg).push(zmq::Message::new());
                    }
                }

                return Ok(Async::Ready(()));
            }
            Async::NotReady => {
                return Ok(Async::NotReady);
            }
        }
    }
}
