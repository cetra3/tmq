/// Utility implementations

/// Implements common functions for a Socket wrapper.
macro_rules! impl_wrapper {
    ($type: ty, $target: ty, $field: ident) => {
        impl $crate::socket::AsZmqSocket for $type {
            #[inline]
            fn get_socket(&self) -> &::zmq::Socket {
                &self.$field.get_socket()
            }
        }
        impl $type {
            #[doc(hidden)]
            #[inline]
            pub fn into_inner(self) -> $target {
                self.$field
            }
        }
    };
}

/// Implements `Sink<Multipart, Error=TmqError>` for a Socket wrapping an inner `Sender`.
macro_rules! impl_wrapper_sink {
    ($type: ty, $field: ident) => {
        impl<T: ::std::convert::Into<$crate::Multipart>> ::futures::Sink<T> for $type {
            type Error = $crate::TmqError;

            #[inline]
            fn poll_ready(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                ::futures::Sink::<T>::poll_ready(::std::pin::Pin::new(&mut self.$field), cx)
            }

            #[inline]
            fn start_send(mut self: ::std::pin::Pin<&mut Self>, item: T) -> $crate::Result<()> {
                ::futures::Sink::<T>::start_send(::std::pin::Pin::new(&mut self.$field), item)
            }

            #[inline]
            fn poll_flush(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                ::futures::Sink::<T>::poll_flush(::std::pin::Pin::new(&mut self.$field), cx)
            }

            #[inline]
            fn poll_close(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                ::futures::Sink::<T>::poll_close(::std::pin::Pin::new(&mut self.$field), cx)
            }
        }
    };
}

/// Implements `Stream<Item=Result<Multipart>` for a Socket wrapping an inner `Receiver`.
macro_rules! impl_wrapper_stream {
    ($type: ty, $field: ident) => {
        impl ::futures::Stream for $type {
            type Item = $crate::Result<$crate::Multipart>;

            #[inline]
            fn poll_next(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context,
            ) -> ::std::task::Poll<::std::option::Option<Self::Item>> {
                ::std::pin::Pin::new(&mut self.$field).poll_next(cx)
            }
        }
    };
}

/// Implements AsZmqSocket for the given type.
/// field: identifier of a field containing a `ZmqPoller`.
macro_rules! impl_as_socket {
    ($type: ty, $field: ident) => {
        impl $crate::socket::AsZmqSocket for $type {
            #[inline]
            fn get_socket(&self) -> &::zmq::Socket {
                &self.$field.get_socket()
            }
        }
    };
}

/// Implements a `split` method for a Socket wrapping an inner `SenderReceiver`.
macro_rules! impl_split {
    ($type: ty, $field: ident) => {
        impl $type {
            #[inline]
            pub fn split(self) -> ($crate::SharedReceiver, $crate::SharedSender) {
                self.$field.split()
            }
        }
    };
}

/// Implements a `buffered` method for a Socket wrapping an inner `Receiver`.
macro_rules! impl_buffered {
    ($type: ty, $field: ident) => {
        impl $type {
            pub fn buffered(self, capacity: usize) -> $crate::BufferedReceiver {
                self.$field.buffered(capacity)
            }
        }
    };
}

/// Async read/write implementations
/// Implements Sink<T: Into<Multipart>> for the given type.
/// $buffer: identifier of a field containing a `Multipart`
/// $socket: identifier of a field containing a `ZmqPoller`
macro_rules! impl_sink {
    ($type: ty, $buffer: ident, $socket: ident) => {
        impl<T: ::std::convert::Into<$crate::Multipart>> ::futures::Sink<T> for $type {
            type Error = $crate::TmqError;

            fn poll_ready(
                self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                let Self {
                    ref $socket,
                    ref mut $buffer,
                } = self.get_mut();
                ::futures::ready!($socket.multipart_flush(cx, $buffer))?;
                ::std::task::Poll::Ready($crate::Result::Ok(()))
            }

            fn start_send(mut self: ::std::pin::Pin<&mut Self>, item: T) -> $crate::Result<()> {
                assert!(self.$buffer.is_empty());
                self.$buffer = item.into();
                $crate::Result::Ok(())
            }

            fn poll_flush(
                self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                let Self {
                    ref $socket,
                    ref mut $buffer,
                } = self.get_mut();
                $socket.multipart_flush(cx, $buffer)
            }

            fn poll_close(
                self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<$crate::Result<()>> {
                ::futures::Sink::<T>::poll_flush(self, cx)
            }
        }
    };
}

/// Implements `Stream<Item=Result<Multipart>>` for the given type.
/// $socket: identifier of a field containing a `ZmqPoller`
macro_rules! impl_stream {
    ($type: ty, $socket: ident) => {
        impl ::futures::Stream for $type {
            type Item = $crate::Result<$crate::Multipart>;

            fn poll_next(
                self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context,
            ) -> ::std::task::Poll<::std::option::Option<Self::Item>> {
                match self.$socket.multipart_recv(cx) {
                    ::std::task::Poll::Ready(value) => {
                        ::std::task::Poll::Ready(::std::option::Option::Some(value))
                    }
                    _ => ::std::task::Poll::Pending,
                }
            }
        }
    };
}
/// Implements a buffered `Stream<Item=Result<Multipart>>` for the given type.
/// $buffer: identifier of a field containing a `ReceiveBuffer`
/// $socket: identifier of a field containing a `ZmqPoller`
macro_rules! impl_buffered_stream {
    ($type: ty, $buffer: ident, $socket: ident) => {
        impl ::futures::Stream for $type {
            type Item = $crate::Result<$crate::Multipart>;

            fn poll_next(
                self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context,
            ) -> ::std::task::Poll<::std::option::Option<Self::Item>> {
                let Self {
                    ref $socket,
                    ref mut $buffer,
                } = self.get_mut();
                match $socket.multipart_recv_buffered(cx, $buffer) {
                    ::std::task::Poll::Ready(value) => {
                        ::std::task::Poll::Ready(::std::option::Option::Some(value))
                    }
                    _ => ::std::task::Poll::Pending,
                }
            }
        }
    };
}

/// Builder implementations

/// Creates a builder structure named $builder with methods to bind and connect using the given $socket_type.
/// The $result_type must have a constructor accepting a single field called `socket` of type [`zmq::Socket`].
macro_rules! impl_builder {
    ($socket_type: ident, $builder: ident, $result_type: tt) => {
        /// Builder which provides [`bind`](#method.bind) and [`connect`](#method.connect) methods to prepare a ZMQ socket for creation.
        pub struct $builder<'a> {
            #[doc(hidden)]
            context: &'a ::zmq::Context,
        }

        impl<'a> $builder<'a> {
            #[doc(hidden)]
            fn new(context: &'a ::zmq::Context) -> Self {
                Self { context }
            }

            /// Connect to a ZMQ endpoint at the given address.
            pub fn connect(self, endpoint: &str) -> $crate::Result<$result_type> {
                let socket = self.context.socket(::zmq::SocketType::$socket_type)?;
                socket.connect(endpoint)?;

                $crate::Result::Ok($result_type {
                    socket: socket.into(),
                })
            }
            /// Bind to a ZMQ endpoint at the given address.
            pub fn bind(self, endpoint: &str) -> $crate::Result<$result_type> {
                let socket = self.context.socket(::zmq::SocketType::$socket_type)?;
                socket.bind(endpoint)?;

                $crate::Result::Ok($result_type {
                    socket: socket.into(),
                })
            }
        }
    };
}
