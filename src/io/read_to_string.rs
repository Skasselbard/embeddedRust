use super::read_to_end::read_to_end_internal;
use super::{AsyncRead, Error, Result};
use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::{mem, str};

/// Future for the [`read_to_string`](super::AsyncReadExt::read_to_string) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadToString<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut String,
    bytes: Vec<u8>,
    start_len: usize,
}

impl<R: ?Sized + Unpin> Unpin for ReadToString<'_, R> {}

impl<'a, R: AsyncRead + ?Sized + Unpin> ReadToString<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut String) -> Self {
        let start_len = buf.len();
        Self {
            reader,
            bytes: mem::replace(buf, String::new()).into_bytes(),
            buf,
            start_len,
        }
    }
}

fn read_to_string_internal<R: AsyncRead + ?Sized>(
    reader: Pin<&mut R>,
    cx: &mut Context<'_>,
    buf: &mut String,
    bytes: &mut Vec<u8>,
    start_len: usize,
) -> Poll<Result<usize>> {
    let ret = ready!(read_to_end_internal(reader, cx, bytes, start_len));
    if str::from_utf8(&bytes).is_err() {
        Poll::Ready(ret.and_then(|_| {
            Err(Error::InvalidData)
        }))
    } else {
        debug_assert!(buf.is_empty());
        // Safety: `bytes` is a valid UTF-8 because `str::from_utf8` returned `Ok`.
        mem::swap(unsafe { buf.as_mut_vec() }, bytes);
        Poll::Ready(ret)
    }
}

impl<A> Future for ReadToString<'_, A>
where
    A: AsyncRead + ?Sized + Unpin,
{
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            reader,
            buf,
            bytes,
            start_len,
        } = &mut *self;
        read_to_string_internal(Pin::new(reader), cx, buf, bytes, *start_len)
    }
}
