pub trait Hash {
    fn hash(&self) -> usize;
}

#[derive(Debug)]
struct Slot<K, V>(Option<(K, V)>)
where
    K: Hash + PartialEq;

impl<K, V> Default for Slot<K, V>
where
    K: Hash + PartialEq,
{
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Debug)]
pub struct HashMap<K, V>
where
    K: Hash + PartialEq,
{
    bucket: Vec<Slot<K, V>>,
    current_size: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + PartialEq,
{
    fn resize(&mut self) {
        let load_factor = 7 * self.bucket.len() / 8;
        if self.current_size >= load_factor {
            let new_size = calc_cap(self.bucket.len());
            self.bucket.resize_with(new_size, Default::default);
        }
    }

    fn probe(&self, k: &K) -> Option<&Slot<K, V>> {
        let m = self.bucket.len();
        let probe_index = |i: usize| (k.hash() + (i * i + i) / 2) % m;
        let search = self.bucket[k.hash()..]
            .iter()
            .enumerate()
            .find(|(i, entry)| {
                entry
                    .0
                    .as_ref()
                    .is_none_or(|(k_inner, _)| probe_index(*i) < m && k.eq(&k_inner))
            });
        match search {
            Some((_, slot)) => Some(slot),
            None => unreachable!(),
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        let cap = calc_cap(size);
        let mut bucket = Vec::with_capacity(cap);
        bucket.resize_with(cap, Default::default);

        Self {
            bucket,
            current_size: 0,
        }
    }
    /// Returns the previous item if possible, otherwise None
    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        // let index = k.hash() % self.bucket.len();
        let value = Slot(Some((k, v)));
        self.probe(&k)
            .replace(&value)
            .and_then(|entry| entry.0.take().map(|(_, v)| v))
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.probe(k)
            .and_then(|entry| entry.0.as_ref().map(|(_, v)| v))
    }
}

fn calc_cap(size: usize) -> usize {
    let bucket_size = {
        // buckets smaller than 4 are unusable
        if size < 8 {
            if size < 4 { 4 } else { 8 }
        } else {
            size
        }
    };
    let adjusted = bucket_size * 8 / 7;

    adjusted.next_power_of_two()
}

impl Hash for String {
    fn hash(&self) -> usize {
        self.chars().fold(0, |acc, byte| (acc << 1) ^ byte as usize)
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
    fn test_resizing() {
        let arr = ["Foo", "Bar", "Baz", "Qoux", "Foobar", "Foobaz"];
        let mut map = HashMap::with_capacity(0);
        for (index, input) in arr.iter().enumerate() {
            let (k, v) = (String::from(*input), index);
            map.put(k.clone(), v);
            dbg!(&map);
        }
        for (index, input) in arr.iter().enumerate() {
            let (k, v) = (String::from(*input), index);
            assert_eq!(map.get(&k), Some(v).as_ref());
        }
        assert_eq!(map.get(&String::from("Fositnsio")), None);
    }
}
