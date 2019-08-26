/// Implements AsZmqSocket for the given type.
/// $socket: identifier of a field containing an `EventedSocket`
macro_rules! impl_socket {
    ($type: ty, $socket: ident) => {
        impl crate::socket::AsZmqSocket for $type {
            #[inline]
            fn get_socket(&self) -> &zmq::Socket {
                &self.$socket.get_socket()
            }
        }
    };
}

/// Implements Sink<T: Into<Multipart>> for the given type.
/// $socket: identifier of a field containing an `EventedSocket`
/// $buffer: identifier of af ield containing `Option<Multipart>`
macro_rules! impl_sink {
    ($type: ty, $buffer: ident, $socket: ident) => {
        impl<T: Into<crate::Multipart>> futures::Sink<T> for $type {
            type Error = crate::TmqError;

            fn poll_ready(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                let Self {
                    ref $socket,
                    ref mut $buffer,
                } = self.get_mut();
                futures::ready!($socket.multipart_flush(cx, $buffer))?;
                std::task::Poll::Ready(crate::Result::Ok(()))
            }

            fn start_send(mut self: std::pin::Pin<&mut Self>, item: T) -> crate::Result<()> {
                assert!(self.$buffer.is_empty());
                self.$buffer = item.into();
                crate::Result::Ok(())
            }

            fn poll_flush(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                let Self {
                    ref $socket,
                    ref mut $buffer,
                } = self.get_mut();
                $socket.multipart_flush(cx, $buffer)
            }

            fn poll_close(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                futures::Sink::<T>::poll_flush(self, cx)
            }
        }
    };
}

/// Implements Stream<Multipart> for the given type.
/// $socket: identifier of a field containing an `EventedSocket`
macro_rules! impl_stream {
    ($type: ty, $socket: ident) => {
        impl futures::Stream for $type {
            type Item = crate::Result<crate::Multipart>;

            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context,
            ) -> std::task::Poll<std::option::Option<Self::Item>> {
                self.$socket.multipart_recv(cx)
            }
        }
    };
}

macro_rules! impl_as_buffered {
    ($type: ty, $buffered: ident, $socket: ident) => {
        impl crate::socket::BufferedSocketExt for $type {
            type BufferedStream = $buffered;

            fn buffered_stream(self, capacity: usize) -> Self::BufferedStream {
                $buffered::new(self.$socket, capacity)
            }
        }

        pub struct $buffered {
            $socket: std::rc::Rc<crate::poll::ZmqPoller>,
            buffer: crate::poll::ReceiveBuffer
        }
        impl $buffered {
            pub(crate) fn new(poller: crate::poll::ZmqPoller, capacity: usize) -> Self {
                Self {
                    $socket: std::rc::Rc::new(poller),
                    buffer: crate::poll::ReceiveBuffer::new(capacity)
                }
            }
        }

        impl_buffered_stream!($buffered, buffer, $socket);
    };
}

macro_rules! impl_buffered_stream {
    ($type: ty, $buffer: ident, $socket: ident) => {
        impl futures::Stream for $type {
            type Item = crate::Result<crate::Multipart>;

            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context,
            ) -> std::task::Poll<std::option::Option<Self::Item>> {
                let Self { ref $socket, ref mut $buffer } = self.get_mut();
                $socket.multipart_recv_buffered(cx, $buffer)
            }
        }
    };
}

macro_rules! impl_split {
    ($type: ty, $read: tt, $write: tt, $socket: ident, $buffer: ident) => {
        impl crate::SplitSocketExt for $type {
            type ReadHalf = $read;
            type WriteHalf = $write;

            fn split_socket(self) -> (Self::ReadHalf, Self::WriteHalf) {
                let rc = std::rc::Rc::new(self.$socket);
                (
                    Self::ReadHalf {
                        $socket: rc.clone(),
                    },
                    Self::WriteHalf {
                        $socket: rc,
                        $buffer: self.$buffer,
                    },
                )
            }
        }

        pub struct $read {
            $socket: std::rc::Rc<crate::poll::ZmqPoller>,
        }
        impl_stream!($read, socket);

        pub struct $write {
            $socket: std::rc::Rc<crate::poll::ZmqPoller>,
            $buffer: crate::Multipart,
        }
        impl_sink!($write, $buffer, $socket);
    };
}

/// Implements a connect builder method with the given $socket_type.
/// The type on which the method is implemented should have a field `context` of type
/// `&zmq::Context`.
///
/// Returns a `Result<$result_type>`. The result type should have a single field `socket` of
/// type `zmq::Socket`.
macro_rules! build_connect {
    ($socket_type: ident, $result_type: tt) => {
        pub fn connect(self, endpoint: &str) -> crate::Result<$result_type> {
            let socket = self.context.socket(zmq::SocketType::$socket_type)?;
            socket.connect(endpoint)?;

            crate::Result::Ok($result_type {
                socket: socket.into(),
            })
        }
    }
}

/// Implements a bind builder method with the given $socket_type.
/// The type on which the method is implemented should have a field `context` of type
/// `&zmq::Context`.
///
/// Returns a `Result<$result_type>`. The result type should have a single field `socket` of
/// type `zmq::Socket`.
macro_rules! build_bind {
    ($socket_type: ident, $result_type: tt) => {
        pub fn bind(self, endpoint: &str) -> crate::Result<$result_type> {
            let socket = self.context.socket(zmq::SocketType::$socket_type)?;
            socket.bind(endpoint)?;

            crate::Result::Ok($result_type {
                socket: socket.into(),
            })
        }
    }
}
