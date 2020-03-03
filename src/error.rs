use thiserror::Error;

/// Error that can occur during an async ZMQ operation.
#[derive(Error, Debug)]
pub enum TmqError {
    /// Inner ZMQ error.
    #[error("Zmq error: {0}")]
    Zmq(#[from] zmq::Error),
    /// A send operation of a multipart message was successfully started, but it could not be finished.
    #[error("Interrupted Zmq send. Please report this at https://github.com/cetra3/tmq/issues")]
    InterruptedSend,
    /// General IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
