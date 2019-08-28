use std::{
    collections::VecDeque,
    iter::FromIterator,
    ops::{Index, IndexMut},
};
use zmq::Message;

#[derive(Debug)]
pub struct Multipart(VecDeque<Message>);

impl Multipart {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn push_front(&mut self, item: Message) {
        self.0.push_front(item)
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<Message> {
        self.0.pop_front()
    }

    #[inline]
    pub fn push_back(&mut self, item: Message) {
        self.0.push_back(item)
    }

    #[inline]
    pub fn iter(&self) -> std::collections::vec_deque::Iter<Message> {
        self.0.iter()
    }
}

impl Default for Multipart {
    fn default() -> Self {
        Self(VecDeque::default())
    }
}

impl From<Vec<Message>> for Multipart {
    fn from(item: Vec<Message>) -> Self {
        Self(item.into())
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
