use super::{AsyncWrite, Result};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Future for the [`flush`](super::AsyncWriteExt::flush) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Flush<'a, W: ?Sized> {
    writer: &'a mut W,
}

impl<W: ?Sized + Unpin> Unpin for Flush<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> Flush<'a, W> {
    pub(super) fn new(writer: &'a mut W) -> Self {
        Flush { writer }
    }
}

impl<W> Future for Flush<'_, W>
where
    W: AsyncWrite + ?Sized + Unpin,
{
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.writer).poll_flush(cx)
    }
}
