use std::collections::VecDeque;
use zmq::Message;

pub type Multipart = VecDeque<Message>;
