use std::io::Result;

pub trait WriteItems<T> {
    fn write_items(&mut self, items: &[T]) -> Result<usize>;
}

pub trait ReadItems<T> {
    fn read_items(&mut self, items: &mut [T]) -> Result<usize>;
}

pub trait ItemsAvailable<T> {
    fn items_available(&self) -> Result<usize>;
}
