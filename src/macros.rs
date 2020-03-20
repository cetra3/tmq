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

macro_rules! impl_buffered {
    ($type: ty, $field: ident) => {
        impl $type {
            /// Implements a `buffered` method for the Socket wrapping an inner `Receiver`.
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
