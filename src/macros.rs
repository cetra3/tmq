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
    ($type: ty, $socket: ident) => {
        impl<T: Into<crate::Multipart>> futures::Sink<T> for $type {
            type Error = crate::TmqError;

            fn poll_ready(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                futures::ready!(self.$socket.multipart_flush(cx))?;
                self.$socket.multipart_poll_ready(cx)
            }

            fn start_send(mut self: std::pin::Pin<&mut Self>, item: T) -> crate::Result<()> {
                self.$socket.multipart_set_buffer(item.into());
                crate::Result::Ok(())
            }

            fn poll_flush(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                self.$socket.multipart_flush(cx)
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
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context,
            ) -> std::task::Poll<std::option::Option<Self::Item>> {
                self.$socket.multipart_recv(cx)
            }
        }
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
