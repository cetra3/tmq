quick_error! {
    #[derive(Debug)]
    pub enum TmqError {
        Zmq(err: zmq::Error) {
            from()
        }
        Io(err: std::io::Error) {
            from()
        }
    }
}
