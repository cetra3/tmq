quick_error! {
    #[derive(Debug)]
    pub enum TmqError {
        Zmq(err: zmq::Error) {
            from()
            cause(err)
            description(err.description())
        }
        Io(err: std::io::Error) {
            from()
            cause(err)
            description(err.description())
        }
    }
}
