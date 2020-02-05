use thiserror::Error;

/// Error that can occur during an async ZMQ operation.
#[derive(Error, Debug)]
pub enum TmqError {
    /// Inner ZMQ error.
    #[error("Zmq error: {0}")]
    Zmq(#[from] zmq::Error),
    /// General IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
