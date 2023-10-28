use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;

use hashbrown::HashTable;

pub struct Id<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> Id<T> {
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }

    pub const fn get(&self) -> usize {
        self.index
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.index).finish()
    }
}

pub struct IndexTable<T> {
    pub values: Vec<T>,
    pub table: HashTable<Id<T>>,
}

fn fxhash_one<T: Hash>(value: &T) -> u64 {
    use std::hash::Hasher;
    let mut hasher = rustc_hash::FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

impl<T> Default for IndexTable<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            table: HashTable::new(),
        }
    }
}

impl<T: Hash + Eq> IndexTable<T> {
    pub fn get_or_insert(&mut self, value: T) -> Id<T> {
        let hash = fxhash_one(&value);
        match self
            .table
            .find_entry(hash, |idx| self.values[idx.get()] == value)
        {
            Ok(entry) => *entry.get(),
            Err(entry) => {
                let id = Id::new(self.values.len());
                self.values.push(value);
                entry.into_table().insert_unique(hash, id, fxhash_one);
                id
            }
        }
    }
}

impl<T> Index<Id<T>> for IndexTable<T> {
    type Output = T;

    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.values[index.get()]
    }
}
