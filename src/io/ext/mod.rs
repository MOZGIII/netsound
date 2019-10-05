use super::*;

mod write_items;
pub use write_items::*;

mod read_items;
pub use read_items::*;

mod read_exact_items;
pub use read_exact_items::*;

mod items_available;
pub use items_available::*;

mod wait_for_items_available;
pub use wait_for_items_available::*;

pub trait AsyncWriteItemsExt<T: Unpin>: AsyncWriteItems<T> {
    fn write_items<'a>(&'a mut self, buf: &'a [T]) -> WriteItems<'a, T, Self>
    where
        Self: Unpin,
    {
        WriteItems::new(self, buf)
    }
}

impl<T: Unpin, W: AsyncWriteItems<T> + ?Sized> AsyncWriteItemsExt<T> for W {}

pub trait AsyncReadItemsExt<T: Unpin>: AsyncReadItems<T> {
    fn read_items<'a>(&'a mut self, buf: &'a mut [T]) -> ReadItems<'a, T, Self>
    where
        Self: Unpin,
    {
        ReadItems::new(self, buf)
    }

    fn read_exact_items<'a>(&'a mut self, buf: &'a mut [T]) -> ReadExactItems<'a, T, Self>
    where
        Self: Unpin,
    {
        ReadExactItems::new(self, buf)
    }
}

impl<T: Unpin, R: AsyncReadItems<T> + ?Sized> AsyncReadItemsExt<T> for R {}

pub trait AsyncItemsAvailableExt<T: Unpin>: AsyncItemsAvailable<T> {
    fn items_available<'a>(&'a mut self) -> ItemsAvailable<'a, T, Self>
    where
        Self: Unpin,
    {
        ItemsAvailable::new(self)
    }

    fn wait_for_items_available<'a>(
        &'a mut self,
        required_items: usize,
    ) -> WaitForItemsAvailable<'a, T, Self>
    where
        Self: Unpin,
    {
        WaitForItemsAvailable::new(self, required_items)
    }
}

impl<T: Unpin, P: AsyncItemsAvailable<T> + ?Sized> AsyncItemsAvailableExt<T> for P {}
