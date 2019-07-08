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
    fn send_message(&self, msg: &TmqMessage) -> Poll<(), Error> {
        match self.poll_write_ready()? {
            Async::Ready(_) => {
                match msg {
                    TmqMessage::Single(ref msg) => {
                        //Send the message, and if it will block, then we set up a notifier
                        if let Err(e) = self.get_ref().io.send(&**msg, zmq::DONTWAIT) {
                            if e == zmq::Error::EAGAIN {
                                self.clear_write_ready()?;
                                return Ok(Async::NotReady);
                            } else {
                                return Err(e.into());
                            }
                        }
                    }
                    TmqMessage::Multipart(ref msgs) => {
                        let len = msgs.len();

                        for (i, msg) in msgs.iter().enumerate() {
                            debug!("i:{}, len:{}", i, len);
                            let flags = if i < len - 1 {
                                debug!("Using SNDMORE flag");
                                zmq::DONTWAIT | zmq::SNDMORE
                            } else {
                                debug!("Not using SNDMORE flag");
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
                    }
                };

                return Ok(Async::Ready(()));
            }

            //If it's not ready to send yet, then we basically need to hold onto the message and wait
            Async::NotReady => {
                return Ok(Async::NotReady);
            }
        }
    }

    fn recv_message(&self, msg: &mut TmqMessage) -> Poll<(), Error> {
        let ready = Ready::readable();

        match self.poll_read_ready(ready)? {
            Async::Ready(_) => {
                match msg {
                    TmqMessage::Single(ref mut msg) => {
                        debug!("Single mode!");
                        if let Err(e) = self.get_ref().io.recv(msg, zmq::DONTWAIT) {
                            if e == zmq::Error::EAGAIN {
                                self.clear_read_ready(ready)?;
                                return Ok(Async::NotReady);
                            } else {
                                return Err(e.into());
                            }
                        }
                    }
                    TmqMessage::Multipart(ref mut msgs) => loop {
                        debug!("Multipart mode!");
                        let mut msg = msgs.pop().unwrap_or_else(|| zmq::Message::new());

                        if let Err(e) = self.get_ref().io.recv(&mut msg, zmq::DONTWAIT) {
                            if e == zmq::Error::EAGAIN {
                                self.clear_read_ready(ready)?;
                                msgs.push(msg);
                                return Ok(Async::NotReady);
                            } else {
                                return Err(e.into());
                            }
                        }

                        msgs.push(msg);

                        if !self.get_ref().io.get_rcvmore()? {
                            break;
                        } else {
                            msgs.push(zmq::Message::new());
                        }
                    },
                }

                return Ok(Async::Ready(()));
            }
            Async::NotReady => {
                return Ok(Async::NotReady);
            }
        }
    }
}
