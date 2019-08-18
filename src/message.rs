use zmq::Message;
use std::collections::VecDeque;

pub type Multipart = VecDeque<Message>;
