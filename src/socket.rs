use crate::Result;

use std::os::unix::io::{AsRawFd, RawFd};

/// Wrapper on top of a ZMQ socket.
///
/// The socket needs to be wrapped to allow various trait implementations.
pub(crate) struct SocketWrapper {
    pub(crate) socket: zmq::Socket,
    // This RawFd is held separately because it must be accessible
    // without error after SocketWrapper initialization, for the AsRawFd trait.
    fd: RawFd,
}

impl SocketWrapper {
    pub fn new(socket: zmq::Socket) -> Result<Self> {
        Ok(Self {
            fd: socket.get_fd()?,
            socket,
        })
    }
}

impl AsRawFd for SocketWrapper {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

/// Trait for various ZMQ socket wrappers.
pub trait AsZmqSocket {
    /// Return a reference to the inner ZMQ socket.
    fn get_socket(&self) -> &zmq::Socket;
}

/// Trait which defines configuration functions for ZMQ sockets.
///
/// See ZMQ documentation for more info: [http://api.zeromq.org/4-2:zmq-setsockopt](http://api.zeromq.org/4-2:zmq-setsockopt)
///
/// Some socket options need to be set before bind/connect via the [`SocketBuilder`](struct.SocketBuilder.html) methods
pub trait SocketExt {
    /// Accessor for the `ZMQ_LINGER` option.
    fn get_linger(&self) -> Result<i32>;
    /// Setter for the `ZMQ_LINGER` option.
    fn set_linger(&self, value: i32) -> Result<()>;

    /// Accessor for the `ZMQ_SNDHWM` option.
    fn get_sndhwm(&self) -> Result<i32>;
    /// Setter for the `ZMQ_SNDHWM` option.
    fn set_sndhwm(&self, value: i32) -> Result<()>;

    /// Accessor for the `ZMQ_RCVHWM` option.
    fn get_rcvhwm(&self) -> Result<i32>;
    /// Setter for the `ZMQ_RCVHWM` option.
    fn set_rcvhwm(&self, value: i32) -> Result<()>;

    /// Accessor for the `ZMQ_PROBE_ROUTER` option.
    fn is_probe_router(&self) -> Result<bool>;
    /// Setter for the `ZMQ_PROBE_ROUTER` option.
    fn set_probe_router(&self, value: bool) -> Result<()>;

    /// Accessor for the `ZMQ_IPV6` option.
    fn is_ipv6(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_IMMEDIATE` option.
    fn is_immediate(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_PLAIN_SERVER` option.
    fn is_plain_server(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_CONFLATE` option.
    fn is_conflate(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_CURVE_SERVER` option.
    fn is_curve_server(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_GSSAPI_SERVER` option.
    fn is_gssapi_server(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_GSSAPI_PLAINTEXT` option.
    fn is_gssapi_plaintext(&self) -> Result<bool>;

    /// Accessor for the `ZMQ_MAXMSGSIZE` option.
    fn get_maxmsgsize(&self) -> Result<i64>;

    /// Accessor for the `ZMQ_AFFINITY` option.
    fn get_affinity(&self) -> Result<u64>;

    /// Accessor for the `ZMQ_RATE` option.
    fn get_rate(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_RECOVERY_IVL` option.
    fn get_recovery_ivl(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_SNDBUF` option.
    fn get_sndbuf(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_RCVBUF` option.
    fn get_rcvbuf(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_TOS` option.
    fn get_tos(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_RECONNECT_IVL` option.
    fn get_reconnect_ivl(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_RECONNECT_IVL_MAX` option.
    fn get_reconnect_ivl_max(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_BACKLOG` option.
    fn get_backlog(&self) -> Result<i32>;

    /// Accessor for the `ZMQ_IDENTITY` option.
    fn get_identity(&self) -> Result<Vec<u8>>;
}

macro_rules! getter {
    ($name: ident, $retval: ty) => {
        #[inline]
        fn $name(&self) -> $crate::Result<$retval> {
            self.get_socket().$name().map_err(|e| e.into())
        }
    };
}
macro_rules! setter {
    ($name: ident, $type: ty) => {
        #[inline]
        fn $name(&self, value: $type) -> $crate::Result<()> {
            self.get_socket().$name(value).map_err(|e| e.into())
        }
    };
}

impl<T: AsZmqSocket> SocketExt for T {
    getter!(is_ipv6, bool);

    getter!(is_immediate, bool);

    getter!(is_plain_server, bool);

    getter!(is_conflate, bool);

    getter!(is_probe_router, bool);
    setter!(set_probe_router, bool);

    getter!(is_curve_server, bool);

    getter!(is_gssapi_server, bool);

    getter!(is_gssapi_plaintext, bool);

    getter!(get_maxmsgsize, i64);

    getter!(get_sndhwm, i32);
    setter!(set_sndhwm, i32);

    getter!(get_rcvhwm, i32);
    setter!(set_rcvhwm, i32);

    getter!(get_affinity, u64);

    getter!(get_rate, i32);

    getter!(get_recovery_ivl, i32);

    getter!(get_sndbuf, i32);

    getter!(get_rcvbuf, i32);

    getter!(get_tos, i32);

    getter!(get_linger, i32);
    setter!(set_linger, i32);

    getter!(get_reconnect_ivl, i32);

    getter!(get_reconnect_ivl_max, i32);

    getter!(get_backlog, i32);

    getter!(get_identity, Vec<u8>);
}
