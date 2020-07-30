use std::future::Future;
use std::io;
use std::task::Poll;

use futures_lite::{future, pin};

// ref https://github.com/stjepang/async-io/blob/v0.1.7/src/lib.rs#L1276-L1289
pub(crate) async fn optimistic(fut: impl Future<Output = io::Result<()>>) -> io::Result<()> {
    let mut polled = false;
    pin!(fut);

    future::poll_fn(|cx| {
        if !polled {
            polled = true;
            fut.as_mut().poll(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    })
    .await
}
