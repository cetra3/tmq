use thiserror::Error;

#[derive(Error, Debug)]
pub enum TmqError {
    #[error("Zmq error: {0}")]
    Zmq(#[from] zmq::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error)
}
