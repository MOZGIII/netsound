use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Copy, Clone)]
pub struct UnorderedEq<V, T>
where
    V: AsRef<[T]>,
{
    value_type: std::marker::PhantomData<T>,
    values: V,
}

impl<T, V> UnorderedEq<V, T>
where
    V: AsRef<[T]>,
    T: Eq + Hash,
{
    pub fn new(values: V) -> Self {
        Self {
            value_type: std::marker::PhantomData,
            values,
        }
    }

    fn count_map(&self) -> HashMap<&T, usize> {
        let mut cnt = HashMap::new();
        for i in self.values.as_ref() {
            *cnt.entry(i).or_insert(0) += 1
        }
        cnt
    }
}

impl<T, V> PartialEq for UnorderedEq<V, T>
where
    V: AsRef<[T]>,
    T: Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.count_map() == other.count_map()
    }
}
