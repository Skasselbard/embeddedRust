use super::{AsyncWrite, Error, Result};
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Future for the [`write_all`](super::AsyncWriteExt::write_all) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteAll<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

impl<W: ?Sized + Unpin> Unpin for WriteAll<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> WriteAll<'a, W> {
    pub(super) fn new(writer: &'a mut W, buf: &'a [u8]) -> Self {
        WriteAll { writer, buf }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteAll<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = &mut *self;
        while !this.buf.is_empty() {
            let n = ready!(Pin::new(&mut this.writer).poll_write(cx, this.buf))?;
            {
                let (_, rest) = mem::replace(&mut this.buf, &[]).split_at(n);
                this.buf = rest;
            }
            if n == 0 {
                return Poll::Ready(Err(Error::WriteZero));
            }
        }

        Poll::Ready(Ok(()))
    }
}
