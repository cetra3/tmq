/// Implements Sink<T: Into<Multipart>> for the given type.
/// $socket: identifier of a field containing an `EventedSocket`
/// $buffer: identifier of af ield containing `Option<Multipart>`
macro_rules! impl_sink {
    ($type: ty, $socket: ident, $buffer:ident) => {
        impl<T: Into<crate::Multipart>> futures::Sink<T> for $type {
            type Error = crate::TmqError;

            fn poll_ready(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                let buf = self.$buffer.take();
                let (poll, buffer) = self.$socket.multipart_flush(cx, buf, true);
                self.$buffer = buffer;
                poll
            }

            fn start_send(mut self: std::pin::Pin<&mut Self>, item: T) -> crate::Result<()> {
                assert_eq!(self.buffer, None);
                self.$buffer = Some(item.into());
                Ok(())
            }

            fn poll_flush(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<crate::Result<()>> {
                let buf = self.$buffer.take();
                let (poll, buffer) = self.$socket.multipart_flush(cx, buf, false);
                self.$buffer = buffer;
                poll
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
