use super::{AsyncRead, Error, Result};
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Future for the [`read_exact`](super::AsyncReadExt::read_exact) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadExact<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}

impl<R: ?Sized + Unpin> Unpin for ReadExact<'_, R> {}

impl<'a, R: AsyncRead + ?Sized + Unpin> ReadExact<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        ReadExact { reader, buf }
    }
}

impl<R: AsyncRead + ?Sized + Unpin> Future for ReadExact<'_, R> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        while !this.buf.is_empty() {
            let n = ready!(Pin::new(&mut this.reader).poll_read(cx, this.buf))?;
            {
                let (_, rest) = mem::replace(&mut this.buf, &mut []).split_at_mut(n);
                this.buf = rest;
            }
            if n == 0 {
                return Poll::Ready(Err(Error::UnexpectedEof));
            }
        }
        Poll::Ready(Ok(()))
    }
}
