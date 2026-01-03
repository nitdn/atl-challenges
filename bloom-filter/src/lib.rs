use std::{
    hash::{BuildHasher, Hash, RandomState},
    iter,
};

#[derive(Debug)]
pub struct BloomFilter {
    hashers: Vec<RandomState>,
    field: Vec<bool>,
}

impl BloomFilter {
    pub fn new(m: usize, k: usize) -> Self {
        Self {
            hashers: iter::repeat_with(RandomState::new).take(k).collect(),
            field: vec![false; m],
        }
    }

    pub fn insert(&mut self, value: &impl Hash) {
        for hasher in &self.hashers {
            let index = hasher.hash_one(value) as usize % self.field.len();
            self.field[index] = true;
        }
    }

    pub fn query(&self, value: &impl Hash) -> bool {
        self.hashers.iter().all(|hasher| {
            let index = hasher.hash_one(value) as usize % self.field.len();
            self.field.get(index).is_some_and(|&pred| pred)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_negative() {
        let mut filter = BloomFilter::new(100_000, 3);
        for value in 0..50_000 {
            filter.insert(&value);
            assert!(filter.query(&value))
        }
    }

    #[test]
    fn test_false_positive() {
        let mut filter = BloomFilter::new(50, 3);
        for value in 0..50 {
            filter.insert(&value);
        }
        assert!((50..100).any(|value| filter.query(&value)));
        assert!(!(50..100).all(|value| filter.query(&value)))
    }
}
