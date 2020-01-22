use std::io;

use mio::{unix::EventedFd, Evented, Poll, PollOpt, Ready, Token};

use crate::Result;

/// Wrapper on top of a ZMQ socket.
///
/// The socket needs to be wrapped to allow various trait implementations.
pub(crate) struct SocketWrapper {
    pub(crate) socket: zmq::Socket,
}

impl SocketWrapper {
    pub fn new(socket: zmq::Socket) -> Self {
        Self { socket }
    }
}

impl Evented for SocketWrapper {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).register(poll, token, interest, opts)
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).reregister(poll, token, interest, opts)
    }
    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.socket.get_fd()?).deregister(poll)
    }
}

/// Trait for various ZMQ socket wrappers.
pub trait AsZmqSocket {
    /// Return a reference to the inner ZMQ socket.
    fn get_socket(&self) -> &zmq::Socket;
}

/// Trait which defines configuration functions for ZMQ sockets.
pub trait SocketExt {
    fn monitor(&self, monitor_endpoint: &str, events: i32) -> Result<()>;

    fn is_ipv6(&self) -> Result<bool>;
    fn set_ipv6(&self, value: bool) -> Result<()>;

    fn is_immediate(&self) -> Result<bool>;
    fn set_immediate(&self, value: bool) -> Result<()>;

    fn is_plain_server(&self) -> Result<bool>;
    fn set_plain_server(&self, value: bool) -> Result<()>;

    fn is_conflate(&self) -> Result<bool>;
    fn set_conflate(&self, value: bool) -> Result<()>;

    fn is_probe_router(&self) -> Result<bool>;
    fn set_probe_router(&self, value: bool) -> Result<()>;

    fn is_curve_server(&self) -> Result<bool>;
    fn set_curve_server(&self, value: bool) -> Result<()>;

    // JP: Comment out to allow this to compile on my system where these APIs are not defined.
    // please don't check this in.
    // fn is_gssapi_server(&self) -> Result<bool>;
    // fn set_gssapi_server(&self, value: bool) -> Result<()>;
    
    // fn is_gssapi_plaintext(&self) -> Result<bool>;
    // fn set_gssapi_plaintext(&self, value: bool) -> Result<()>;

    fn get_maxmsgsize(&self) -> Result<i64>;
    fn set_maxmsgsize(&self, value: i64) -> Result<()>;

    fn get_sndhwm(&self) -> Result<i32>;
    fn set_sndhwm(&self, value: i32) -> Result<()>;

    fn get_rcvhwm(&self) -> Result<i32>;
    fn set_rcvhwm(&self, value: i32) -> Result<()>;

    fn get_affinity(&self) -> Result<u64>;
    fn set_affinity(&self, value: u64) -> Result<()>;

    fn get_rate(&self) -> Result<i32>;
    fn set_rate(&self, value: i32) -> Result<()>;

    fn get_recovery_ivl(&self) -> Result<i32>;
    fn set_recovery_ivl(&self, value: i32) -> Result<()>;

    fn get_sndbuf(&self) -> Result<i32>;
    fn set_sndbuf(&self, value: i32) -> Result<()>;

    fn get_rcvbuf(&self) -> Result<i32>;
    fn set_rcvbuf(&self, value: i32) -> Result<()>;

    fn get_tos(&self) -> Result<i32>;
    fn set_tos(&self, value: i32) -> Result<()>;

    fn get_linger(&self) -> Result<i32>;
    fn set_linger(&self, value: i32) -> Result<()>;

    fn get_reconnect_ivl(&self) -> Result<i32>;
    fn set_reconnect_ivl(&self, value: i32) -> Result<()>;

    fn get_reconnect_ivl_max(&self) -> Result<i32>;
    fn set_reconnect_ivl_max(&self, value: i32) -> Result<()>;

    fn get_backlog(&self) -> Result<i32>;
    fn set_backlog(&self, value: i32) -> Result<()>;
}

macro_rules! getter {
    ($name: ident, $retval: ty) => {
        fn $name(&self) -> $crate::Result<$retval> {
            self.get_socket().$name().map_err(|e| e.into())
        }
    }
}
macro_rules! setter {
    ($name: ident, $type: ty) => {
        fn $name(&self, value: $type) -> $crate::Result<()> {
            self.get_socket().$name(value).map_err(|e| e.into())
        }
    }
}

impl<T: AsZmqSocket> SocketExt for T {
    fn monitor(&self, monitor_endpoint: &str, events: i32) -> Result<()> {
        self.get_socket()
            .monitor(monitor_endpoint, events)
            .map_err(|e| e.into())
    }

    getter!(is_ipv6, bool);
    setter!(set_ipv6, bool);

    getter!(is_immediate, bool);
    setter!(set_immediate, bool);

    getter!(is_plain_server, bool);
    setter!(set_plain_server, bool);

    getter!(is_conflate, bool);
    setter!(set_conflate, bool);

    getter!(is_probe_router, bool);
    setter!(set_probe_router, bool);

    getter!(is_curve_server, bool);
    setter!(set_curve_server, bool);

    // getter!(is_gssapi_server, bool);
    // setter!(set_gssapi_server, bool);

    // getter!(is_gssapi_plaintext, bool);
    // setter!(set_gssapi_plaintext, bool);

    getter!(get_maxmsgsize, i64);
    setter!(set_maxmsgsize, i64);

    getter!(get_sndhwm, i32);
    setter!(set_sndhwm, i32);

    getter!(get_rcvhwm, i32);
    setter!(set_rcvhwm, i32);

    getter!(get_affinity, u64);
    setter!(set_affinity, u64);

    getter!(get_rate, i32);
    setter!(set_rate, i32);

    getter!(get_recovery_ivl, i32);
    setter!(set_recovery_ivl, i32);

    getter!(get_sndbuf, i32);
    setter!(set_sndbuf, i32);

    getter!(get_rcvbuf, i32);
    setter!(set_rcvbuf, i32);

    getter!(get_tos, i32);
    setter!(set_tos, i32);

    getter!(get_linger, i32);
    setter!(set_linger, i32);

    getter!(get_reconnect_ivl, i32);
    setter!(set_reconnect_ivl, i32);

    getter!(get_reconnect_ivl_max, i32);
    setter!(set_reconnect_ivl_max, i32);

    getter!(get_backlog, i32);
    setter!(set_backlog, i32);
}
