use super::*;

mod write_items;
pub use write_items::*;

mod read_items;
pub use read_items::*;

mod read_exact_items;
pub use read_exact_items::*;

#[allow(clippy::module_name_repetitions)]
pub trait AsyncWriteItemsExt<T: Unpin>: AsyncWriteItems<T> {
    fn write_items<'a>(&'a mut self, buf: &'a [T], wait_mode: WaitMode) -> WriteItems<'a, T, Self>
    where
        Self: Unpin,
    {
        WriteItems::new(self, buf, wait_mode)
    }
}

impl<T: Unpin, W: AsyncWriteItems<T> + ?Sized> AsyncWriteItemsExt<T> for W {}

#[allow(clippy::module_name_repetitions)]
pub trait AsyncReadItemsExt<T: Unpin>: AsyncReadItems<T> {
    fn read_items<'a>(&'a mut self, buf: &'a mut [T], wait_mode: WaitMode) -> ReadItems<'a, T, Self>
    where
        Self: Unpin,
    {
        ReadItems::new(self, buf, wait_mode)
    }

    fn read_exact_items<'a>(
        &'a mut self,
        buf: &'a mut [T],
        wait_mode: WaitMode,
    ) -> ReadExactItems<'a, T, Self>
    where
        Self: Unpin,
    {
        ReadExactItems::new(self, buf, wait_mode)
    }
}

impl<T: Unpin, R: AsyncReadItems<T> + ?Sized> AsyncReadItemsExt<T> for R {}
