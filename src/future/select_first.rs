use futures::future::{select, Either};
use std::future::Future;

pub async fn select_first<T, A, B>(future1: A, future2: B) -> T
where
    A: Future<Output = T> + Unpin,
    B: Future<Output = T> + Unpin,
{
    match select(future1, future2).await {
        Either::Left((x, _)) => x,
        Either::Right((x, _)) => x,
    }
}
