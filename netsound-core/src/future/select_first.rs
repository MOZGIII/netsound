use futures::future::{select, Either};
use std::future::Future;

pub async fn select_first<T, A, B>(future1: A, future2: B) -> T
where
    A: Future<Output = T> + Unpin,
    B: Future<Output = T> + Unpin,
{
    take_first(select(future1, future2).await)
}

pub fn take_first<T, A, B>(selected: Either<(T, B), (T, A)>) -> T {
    match selected {
        Either::Left((x, _)) | Either::Right((x, _)) => x,
    }
}
