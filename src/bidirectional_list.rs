use std::{collections::HashMap, hash::Hash};

pub struct BidirectionalList<T>
where
    T: Clone + Hash + Eq + 'static,
{
    index: Vec<&'static T>,
    inverse_index: HashMap<T, usize>,
}

impl<T> Default for BidirectionalList<T>
where
    T: Clone + Hash + Eq + 'static,
{
    fn default() -> Self {
        Self {
            index: Default::default(),
            inverse_index: Default::default(),
        }
    }
}

impl<T> BidirectionalList<T>
where
    T: Clone + Hash + Eq + 'static,
{
    pub fn push(&mut self, value: T) -> usize {
        let idx = self.index.len();
        self.index.push(Box::leak(Box::new(value.clone())));
        self.inverse_index.insert(value, idx);

        idx
    }

    pub fn get(&self, idx: usize) -> Option<&'static T> {
        self.index.get(idx).copied()
    }

    pub fn get_index(&self, value: &T) -> Option<usize> {
        self.inverse_index.get(value).cloned()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}
