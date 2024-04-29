use crate::{FromZmqSocket, TmqError};
use zmq::{Context, SocketType};

macro_rules! setter {
    ($name: ident, $type: ty, $doc: expr) => {
        #[doc=$doc]
        pub fn $name(mut self, value: $type) -> Self {
            if self.error.is_some() {
                return self;
            }

            if let Some(ref socket) = self.socket {
                if let Err(err) = socket.$name(value) {
                    self.error = Some(err.into());
                }
            }

            self
        }
    };
}

/// Builder which provides [`bind`] and [`connect`] methods to build a corresponding ZMQ socket as per the standard functions
///
/// You can use the standard functions from the crate to create a builder
///
/// Use the `set_` functions to set socket options before binding/connecting
///
/// See ZMQ documentation for more info on what these options do: [http://api.zeromq.org/4-2:zmq-setsockopt](http://api.zeromq.org/4-2:zmq-setsockopt)
///
/// [`bind`]: struct.SocketBuilder.html#method.bind
/// [`connect`]: struct.SocketBuilder.html#method.connect
pub struct SocketBuilder<T> {
    socket: Option<::zmq::Socket>,
    error: Option<TmqError>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SocketBuilder<T>
where
    T: FromZmqSocket<T>,
{
    #[doc(hidden)]
    pub(crate) fn new(context: &Context, socket_type: SocketType) -> Self {
        let mut socket = None;

        //Defer the Error to make things easier for consumers
        let mut error = None;

        match context.socket(socket_type) {
            Ok(sock) => socket = Some(sock),
            Err(err) => error = Some(err.into()),
        };

        Self {
            socket,
            error,
            _phantom: Default::default(),
        }
    }

    /// Connect to a ZMQ endpoint at the given address.
    pub fn connect(self, endpoint: &str) -> crate::Result<T> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let socket = self.socket.unwrap();
        socket.connect(endpoint)?;
        T::from_zmq_socket(socket)
    }

    /// Bind to a ZMQ endpoint at the given address.
    pub fn bind(self, endpoint: &str) -> crate::Result<T> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let socket = self.socket.unwrap();
        socket.bind(endpoint)?;
        T::from_zmq_socket(socket)
    }

    /// Configure the socket for [monitoring](http://api.zeromq.org/4-2:zmq-socket-monitor)
    pub fn monitor(mut self, monitor_endpoint: &str, events: i32) -> Self {
        if self.error.is_some() {
            return self;
        }

        if let Some(ref socket) = self.socket {
            if let Err(err) = socket.monitor(monitor_endpoint, events) {
                self.error = Some(err.into());
            }
        }

        self
    }

    setter!(set_ipv6, bool, "Setter for the `ZMQ_IPV6` option.");
    setter!(
        set_immediate,
        bool,
        "Setter for the `ZMQ_IMMEDIATE` option."
    );
    setter!(
        set_plain_server,
        bool,
        "Setter for the `ZMQ_PLAIN_SERVER` option."
    );
    setter!(set_conflate, bool, "Setter for the `ZMQ_CONFLATE` option.");
    setter!(
        set_probe_router,
        bool,
        "Setter for the `ZMQ_PROBE_ROUTER` option."
    );
    setter!(
        set_curve_server,
        bool,
        "Setter for the `ZMQ_CURVE_SERVER` option."
    );
    setter!(
        set_curve_secretkey,
        &[u8],
        "Setter for the `ZMQ_CURVE_SECRETKEY` option."
    );
    setter!(
        set_curve_serverkey,
        &[u8],
        "Setter for the `ZMQ_CURVE_SERVERKEY` option."
    );
    setter!(
        set_curve_publickey,
        &[u8],
        "Setter for the `ZMQ_CURVE_PUBLICKEY` option."
    );
    setter!(
        set_gssapi_server,
        bool,
        "Setter for the `ZMQ_GSSAPI_SERVER` option."
    );
    setter!(
        set_gssapi_plaintext,
        bool,
        "Setter for the `ZMQ_GSSAPI_PLAINTEXT` option."
    );
    setter!(
        set_maxmsgsize,
        i64,
        "Setter for the `ZMQ_MAXMSGSIZE` option."
    );
    setter!(set_sndhwm, i32, "Setter for the `ZMQ_SNDHWM` option.");
    setter!(set_rcvhwm, i32, "Setter for the `ZMQ_RCVHWM` option.");
    setter!(set_affinity, u64, "Setter for the `ZMQ_AFFINITY` option.");
    setter!(set_rate, i32, "Setter for the `ZMQ_RATE` option.");
    setter!(
        set_recovery_ivl,
        i32,
        "Setter for the `ZMQ_RECOVERY_IVL` option."
    );
    setter!(set_sndbuf, i32, "Setter for the `ZMQ_SNDBUF` option.");
    setter!(set_rcvbuf, i32, "Setter for the `ZMQ_RCVBUF` option.");
    setter!(set_tos, i32, "Setter for the `ZMQ_TOS` option.");
    setter!(set_linger, i32, "Setter for the `ZMQ_LINGER` option.");
    setter!(
        set_reconnect_ivl,
        i32,
        "Setter for the `ZMQ_RECONNECT_IVL` option."
    );
    setter!(
        set_reconnect_ivl_max,
        i32,
        "Setter for the `ZMQ_RECONNECT_IVL_MAX` option."
    );
    setter!(set_backlog, i32, "Setter for the `ZMQ_BACKLOG` option.");
    setter!(set_identity, &[u8], "Setter for the `ZMQ_IDENTITY` option.");

    setter!(
        set_tcp_keepalive,
        i32,
        "Setter for the `ZMQ_TCP_KEEPALIVE` option."
    );
    setter!(
        set_tcp_keepalive_cnt,
        i32,
        "Setter for the `ZMQ_TCP_KEEPALIVE_CNT` option."
    );
    setter!(
        set_tcp_keepalive_idle,
        i32,
        "Setter for the `ZMQ_TCP_KEEPALIVE_IDLE` option."
    );
    setter!(
        set_tcp_keepalive_intvl,
        i32,
        "Setter for the `ZMQ_TCP_KEEPALIVE_INTVL` option."
    );
    setter!(set_rcvtimeo, i32, "Setter for the `ZMQ_RCVTIMEO` option.");
    setter!(set_sndtimeo, i32, "Setter for the `ZMQ_SNDTIMEO` option.");
}
