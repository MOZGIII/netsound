use std::borrow::Borrow;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub async fn multisend<'a, I>(
    socket: &UdpSocket,
    buf: &[u8],
    peer_addrs: I,
) -> std::io::Result<Vec<usize>>
where
    I: IntoIterator<Item = &'a SocketAddr>,
{
    let peer_addrs_iter = peer_addrs.into_iter();
    let mut sizes = {
        let (lower, upper) = peer_addrs_iter.size_hint();
        Vec::with_capacity(match upper {
            Some(upper) => upper,
            None => lower,
        })
    };

    for peer_addr in peer_addrs_iter {
        let size = socket.send_to(buf, peer_addr).await?;
        sizes.push(size);
    }

    Ok(sizes)
}

pub fn ensure_same_sizes<I, S>(sizes: I) -> Option<usize>
where
    S: Borrow<usize>,
    I: IntoIterator<Item = S>,
{
    let mut iter = sizes.into_iter();
    let size = iter.next()?;
    let size = *size.borrow();
    if iter.all(|e| *e.borrow() == size) {
        return Some(size);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::ensure_same_sizes;

    #[test]
    fn ensure_same_sizes_test() {
        assert_eq!(ensure_same_sizes([1, 1, 1]), Some(1));
        assert_eq!(ensure_same_sizes([] as [usize; 0]), None);
        assert_eq!(ensure_same_sizes([1, 2]), None);
    }
}
