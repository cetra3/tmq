use crate::socket::MioSocket;
use failure::Error;
use futures::{Async, Poll};

use mio::Ready;
use tokio::reactor::PollEvented2;

use zmq;

pub trait Poller {
    fn send_message(&self, msg: &zmq::Message) -> Poll<(), Error>;

    fn recv_message(&self, msg: &mut zmq::Message) -> Poll<(), Error>;
}

impl Poller for PollEvented2<MioSocket> {
    fn send_message(&self, msg: &zmq::Message) -> Poll<(), Error> {
        match self.poll_write_ready()? {
            Async::Ready(_) => {
                //Send the message, and if it will block, then we set up a notifier
                if let Err(e) = self.get_ref().io.send(&**msg, zmq::DONTWAIT) {
                    if e == zmq::Error::EAGAIN {
                        self.clear_write_ready()?;
                        return Ok(Async::NotReady);
                    } else {
                        return Err(e.into());
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

    fn recv_message(&self, msg: &mut zmq::Message) -> Poll<(), Error> {
        let ready = Ready::readable();

        match self.poll_read_ready(ready)? {
            Async::Ready(_) => {
                if let Err(e) = self.get_ref().io.recv(msg, zmq::DONTWAIT) {
                    if e == zmq::Error::EAGAIN {
                        self.clear_read_ready(ready)?;
                        return Ok(Async::NotReady);
                    } else {
                        return Err(e.into());
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
