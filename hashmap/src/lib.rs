pub trait Hash {
    fn hash(&self) -> usize;
}

type Slot<K, V> = Option<(K, V)>;

#[derive(Debug)]
struct Bucket<K, V>(Vec<Slot<K, V>>)
where
    K: Hash + PartialEq;

impl<K: std::ops::Deref, V: std::ops::Deref> std::ops::Deref for Bucket<K, V>
where
    K: Hash + PartialEq,
{
    type Target = Vec<Slot<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct HashMap<K, V>
where
    K: Hash + PartialEq,
{
    bucket: Bucket<K, V>,
    growth_remaining: usize,
    current_size: usize,
}

impl<K, V> Bucket<K, V>
where
    K: Hash + PartialEq,
{
    fn probe(&self, k: &K) -> usize {
        let m = self.0.len();
        for i in 0..m {
            let probe_index = (k.hash() + (i * i + i) / 2) % m;
            if self.0[probe_index]
                .as_ref()
                .is_none_or(|(k_inner, _)| k.eq(k_inner))
            {
                return probe_index;
            }
        }
        unreachable!("We have an overflowing bucket")
    }
    /// This will never create a zero sized bucket
    fn bucket_with_capacity(bucket_len: usize) -> Bucket<K, V> {
        let mut bucket = Vec::with_capacity(bucket_len);
        bucket.resize_with(bucket_len, Default::default);
        Bucket(bucket)
    }

    fn bucket_put(&mut self, k: K, v: V, index: usize) -> Option<V>
    where
        K: Hash + PartialEq,
    {
        let slot = Some((k, v));

        let slot = std::mem::replace(&mut self.0[index], slot);
        slot.map(|(_, v)| v)
    }
}
impl<K, V> HashMap<K, V>
where
    K: Hash + PartialEq + std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn get(&self, k: &K) -> Option<&V> {
        if !self.bucket.0.is_empty() {
            let index = self.bucket.probe(k);
            self.bucket
                .0
                .get(index)
                .and_then(|slot| slot.as_ref().map(|(_, v)| v))
        } else {
            None
        }
    }

    /// Returns the previous item if possible, otherwise None
    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        if self.growth_remaining < 1 {
            self.resize();
        }

        let index = self.bucket.probe(&k);
        if self.bucket.0.get(index).is_some_and(|slot| slot.is_none()) {
            self.current_size += 1;
            self.growth_remaining -= 1;
        }

        self.bucket.bucket_put(k, v, index)
    }
    fn resize(&mut self) {
        let next_bucket_len = calc_cap(self.bucket.0.len() + 1);
        let mut new_bucket = Bucket::bucket_with_capacity(next_bucket_len);
        for slot in self.bucket.0.drain(..) {
            match slot {
                Some((k, v)) => {
                    let index = new_bucket.probe(&k);
                    new_bucket.bucket_put(k, v, index);
                }
                None => continue,
            };
        }
        self.bucket = new_bucket;
        self.growth_remaining = calc_bucket_len(next_bucket_len - 1) - self.current_size;
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let bucket_len = calc_cap(capacity);
        let bucket = Bucket::bucket_with_capacity(bucket_len);
        Self {
            bucket,
            growth_remaining: calc_bucket_len(bucket_len - 1),
            current_size: 0,
        }
    }
}

fn calc_bucket_len(capacity: usize) -> usize {
    // buckets smaller than 8 are not gonna be bothered with
    if capacity < 8 {
        capacity
    } else {
        (capacity + 1) / 8 * 7
    }
}

fn calc_cap(capacity: usize) -> usize {
    // buckets smaller than 4 are unusable
    if capacity < 8 {
        if capacity < 4 { 4 } else { 8 }
    } else {
        let adjusted = capacity * 8 / 7;
        adjusted.next_power_of_two()
    }
}

impl Hash for String {
    fn hash(&self) -> usize {
        self.chars()
            .fold(0, |acc, byte| acc.rotate_left(1) ^ byte as usize)
    }
}

impl Hash for u16 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl Hash for usize {
    fn hash(&self) -> Self {
        *self
    }
}

impl<K: Hash + PartialEq, V> IntoIterator for &mut HashMap<K, V> {
    type Item = (K, V);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.bucket
            .0
            .iter_mut()
            .filter_map(|slot| slot.take())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_get() {
        let (k, v) = (String::from("Foo"), "Bar");
        let mut map = HashMap::with_capacity(10);
        map.put(k.clone(), v);
        assert_eq!(map.get(&k), Some(v).as_ref());
        assert_eq!(map.get(&String::from("Fositnsio")), None);
    }
    #[test]
    fn test_empty() {
        let map: HashMap<String, &'static str> = HashMap::with_capacity(10);
        assert_eq!(map.get(&String::from("foo")), None);
        let map: HashMap<String, &'static str> = HashMap::with_capacity(0);
        assert_eq!(map.get(&String::from("foo")), None);
    }

    #[test]
    fn stress() {
        let mut map = HashMap::with_capacity(10);
        for i in 0..u16::MAX {
            map.put(i, i);
        }
        for i in 0..u16::MAX {
            assert_eq!(map.get(&i), Some(&(i)))
        }
        assert_eq!(map.get(&45832), Some(&(45832)))
    }

    #[test]
    fn test_resizing() {
        let arr = [
            "Foo", "Bar", "Baz", "Qoux", "Foobar", "Foobaz", "Barbar", "Fizz", "Buzz", "Fizzbuzz",
            "Fizzbaz", "FooFiz", "foofiz", "bazfiz",
        ];
        let mut map = HashMap::with_capacity(0);
        for (index, input) in arr.iter().enumerate() {
            for input2 in arr.iter() {
                let mut string_input = String::from(*input);
                string_input.push_str(input2);
                let (k, v) = (string_input, index);
                map.put(k.clone(), v);
            }
        }
        for (index, input) in arr.iter().enumerate() {
            for input2 in arr.iter() {
                let mut string_input = String::from(*input);
                string_input.push_str(input2);
                let (k, v) = (string_input, index);
                assert_eq!(map.get(&k), Some(v).as_ref());
            }
        }
        assert_eq!(map.current_size, arr.len().pow(2));
        assert_eq!(map.get(&String::from("Fositnsio")), None);
    }
}
