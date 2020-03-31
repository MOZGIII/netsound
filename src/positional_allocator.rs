use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;

#[derive(Debug)]
pub struct PositionalAllocator<K, V, G> {
    generator: G,
    map: HashMap<K, V>,
}

impl<K, V, G> PositionalAllocator<K, V, G>
where
    K: Eq + Hash,
    G: Iterator<Item = V>,
{
    pub fn new(generator: G) -> Self {
        let (_lower, upper) = generator.size_hint();
        let map = if let Some(size) = upper {
            HashMap::with_capacity(size)
        } else {
            HashMap::new()
        };
        Self { generator, map }
    }

    pub fn take(&mut self, key: K) -> Option<&V> {
        let map = &mut self.map;
        let generator = &mut self.generator;

        match map.entry(key) {
            Entry::Occupied(e) => Some(e.into_mut()),
            Entry::Vacant(e) => {
                let new_val = generator.next()?;
                Some(e.insert(new_val))
            }
        }
    }

    pub fn allocated(&mut self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }

    pub fn free_all(&mut self, new_generator: G) -> impl Iterator<Item = (K, V)> + '_ {
        self.generator = new_generator;
        self.map.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::PositionalAllocator;

    #[test]
    fn zero_size() {
        let mut map = PositionalAllocator::new(0..0);
        assert!(map.take("a").is_none());
    }

    #[test]
    fn stable_allocation() {
        let mut map = PositionalAllocator::new(0..);
        let val1 = *map.take("a").unwrap();
        let val2 = *map.take("a").unwrap();
        assert_eq!(val1, val2);
    }

    #[test]
    fn generator() {
        let mut map = PositionalAllocator::new(0..2);

        let a = map.take("a").unwrap();
        assert_eq!(*a, 0);

        let b = map.take("b").unwrap();
        assert_eq!(*b, 1);

        assert!(map.take("c").is_none());

        let a = map.take("a").unwrap();
        assert_eq!(*a, 0);
    }

    #[test]
    fn allocated() {
        let mut map = PositionalAllocator::new(0..);
        map.take("a").unwrap();
        map.take("b").unwrap();
        assert_eq!(
            map.allocated().collect::<Vec<_>>(),
            &[(&"a", &0), (&"b", &1)]
        );
    }
}
