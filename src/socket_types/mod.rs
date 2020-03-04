pub mod dealer;
pub mod publish;
pub mod pull;
pub mod push;
pub mod request_reply;
pub mod router;
pub mod subscribe;

pub use dealer::dealer;
pub use publish::publish;
pub use pull::pull;
pub use push::push;
pub use request_reply::reply;
pub use request_reply::request;
pub use router::router;
pub use subscribe::subscribe;

#[doc(hidden)]
pub trait FromZmqSocket<T> {
    fn from_zmq_socket(socket: zmq::Socket) -> crate::Result<T>;
}
