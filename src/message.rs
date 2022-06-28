use std::{
    collections::VecDeque,
    iter::FromIterator,
    ops::{Index, IndexMut},
};
use zmq::Message;

/// ZMQ multipart which holds individual messages.
///
/// It is implemented with a VecDeque to allow efficient popping from the beginning.
/// This is useful both for async Read/Write implementations and for consuming the multipart.
#[derive(Debug, Default, Eq)]
pub struct Multipart(pub VecDeque<Message>);

impl Multipart {
    /// Returns `true` if the multipart contains no messages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of messages in the multipart.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Adds a message to the front of the multipart.
    #[inline]
    pub fn push_front(&mut self, item: Message) {
        self.0.push_front(item)
    }

    /// Removes the first message from the multipart and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop_front(&mut self) -> Option<Message> {
        self.0.pop_front()
    }

    /// Adds a message to the back of the multipart.
    #[inline]
    pub fn push_back(&mut self, item: Message) {
        self.0.push_back(item)
    }

    /// Removes the last message from the multipart and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop_back(&mut self) -> Option<Message> {
        self.0.pop_back()
    }

    /// Creates an iterator which iterates through the messages of this multipart.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Message> {
        self.0.iter()
    }

    /// Creates an iterator mut which iterates through the mutable messages of this multipart.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Message> {
        self.0.iter_mut()
    }
}

impl<T: Into<Message>> From<Vec<T>> for Multipart {
    fn from(item: Vec<T>) -> Self {
        Self(item.into_iter().map(|i| i.into()).collect())
    }
}

impl From<Message> for Multipart {
    fn from(message: Message) -> Self {
        let mut vec = VecDeque::with_capacity(1);
        vec.push_back(message);
        Self(vec)
    }
}

impl FromIterator<Message> for Multipart {
    fn from_iter<T: IntoIterator<Item = Message>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for Multipart {
    type Item = Message;
    type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl PartialEq for Multipart {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Index<usize> for Multipart {
    type Output = Message;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Multipart {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
