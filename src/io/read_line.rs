use super::read_until::read_until_internal;
use super::{AsyncBufRead, Error, Result};
use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::str;
use core::task::{Context, Poll};

/// Future for the [`read_line`](super::AsyncBufReadExt::read_line) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadLine<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut String,
    bytes: Vec<u8>,
    read: usize,
}

impl<R: ?Sized + Unpin> Unpin for ReadLine<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> ReadLine<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut String) -> Self {
        Self {
            reader,
            bytes: mem::replace(buf, String::new()).into_bytes(),
            buf,
            read: 0,
        }
    }
}

pub(super) fn read_line_internal<R: AsyncBufRead + ?Sized>(
    reader: Pin<&mut R>,
    cx: &mut Context<'_>,
    buf: &mut String,
    bytes: &mut Vec<u8>,
    read: &mut usize,
) -> Poll<Result<usize>> {
    let ret = ready!(read_until_internal(reader, cx, b'\n', bytes, read));
    if str::from_utf8(&bytes).is_err() {
        Poll::Ready(ret.and_then(|_| {
            Err(Error::InvalidData)
        }))
    } else {
        debug_assert!(buf.is_empty());
        debug_assert_eq!(*read, 0);
        // Safety: `bytes` is a valid UTF-8 because `str::from_utf8` returned `Ok`.
        mem::swap(unsafe { buf.as_mut_vec() }, bytes);
        Poll::Ready(ret)
    }
}

impl<R: AsyncBufRead + ?Sized + Unpin> Future for ReadLine<'_, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            reader,
            buf,
            bytes,
            read,
        } = &mut *self;
        read_line_internal(Pin::new(reader), cx, buf, bytes, read)
    }
}
