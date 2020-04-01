use super::positional_allocator::PositionalAllocator;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

#[derive(Debug)]
pub struct PreallocatedMap<K, V, S: AsMut<[V]>> {
    value_type: std::marker::PhantomData<V>,
    values: S,
    positional_allocator: PositionalAllocator<K, usize, Range<usize>>,
}

impl<K, V, S> PreallocatedMap<K, V, S>
where
    K: Eq + Hash,
    S: AsMut<[V]>,
{
    pub fn new(mut values: S) -> Self {
        let value_type = std::marker::PhantomData;
        let positional_allocator = PositionalAllocator::new(0..values.as_mut().len());
        Self {
            value_type,
            values,
            positional_allocator,
        }
    }

    pub fn take(&mut self, key: K) -> Option<&mut V> {
        let index = *self.positional_allocator.take(key)?;
        Some(self.values.as_mut().get_mut(index).unwrap())
    }

    pub fn allocated(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let mut allocated: HashMap<&usize, &K> = self
            .positional_allocator
            .allocated()
            .map(|(key, idx)| (idx, key))
            .collect();
        self.values
            .as_mut()
            .iter_mut()
            .enumerate()
            .filter_map(move |(idx, val)| allocated.remove(&idx).map(|key| (key, val)))
    }

    pub fn free_all(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        let new_generator = 0..self.values.as_mut().len();
        let mut freed_values: HashMap<usize, K> = self
            .positional_allocator
            .free_all(new_generator)
            .map(|(key, idx)| (idx, key))
            .collect();
        self.values
            .as_mut()
            .iter_mut()
            .enumerate()
            .filter_map(move |(idx, val)| freed_values.remove(&idx).map(|key| (key, val)))
    }
}

#[cfg(test)]
mod tests {
    use super::PreallocatedMap;

    #[test]
    fn standard_usecase() {
        let values = vec![0, 42];
        let mut map = PreallocatedMap::new(values);

        let val = map.take("a").unwrap();
        assert_eq!(*val, 0);
        *val = 1;

        let val = map.take("b").unwrap();
        assert_eq!(*val, 42);
        *val = 69;

        let val = map.take("a").unwrap();
        assert_eq!(*val, 1);
        *val = 2;

        {
            let mut iter = map.allocated();

            let (key, val) = iter.next().unwrap();
            assert_eq!(key, &"a");
            assert_eq!(*val, 2);

            let (key, val) = iter.next().unwrap();
            assert_eq!(key, &"b");
            assert_eq!(*val, 69);

            assert!(iter.next().is_none());
        }

        {
            let mut iter = map.free_all();

            let (key, val) = iter.next().unwrap();
            assert_eq!(key, "a");
            assert_eq!(*val, 2);

            let (key, val) = iter.next().unwrap();
            assert_eq!(key, "b");
            assert_eq!(*val, 69);

            assert!(iter.next().is_none());
        }

        {
            let mut iter = map.allocated();
            assert!(iter.next().is_none());
        }
    }
}
