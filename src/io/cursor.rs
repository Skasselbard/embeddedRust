use super::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, Error, Result, SeekFrom};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A `Cursor` wraps an in-memory buffer and provides it with a
/// [`AsyncSeek`] implementation.
///
/// `Cursor`s are used with in-memory buffers, anything implementing
/// `AsRef<[u8]>`, to allow them to implement [`AsyncRead`] and/or [`AsyncWrite`],
/// allowing these buffers to be used anywhere you might use a reader or writer
/// that does actual I/O.
///
/// The standard library implements some I/O traits on various types which
/// are commonly used as a buffer, like `Cursor<`[`Vec`]`<u8>>` and
/// `Cursor<`[`&[u8]`][bytes]`>`.
///
/// [`AsyncSeek`]: trait.AsyncSeek.html
/// [`AsyncRead`]: trait.AsyncRead.html
/// [`AsyncWrite`]: trait.AsyncWrite.html
/// [bytes]: https://doc.rust-lang.org/std/primitive.slice.html
#[derive(Clone, Debug, Default)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    ///
    /// Cursor initial position is `0` even if underlying buffer (e.g., `Vec`)
    /// is not empty. So writing to cursor starts with overwriting `Vec`
    /// content, not with appending to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    /// ```
    pub fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }

    /// Consumes this cursor, returning the underlying value.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let vec = buff.into_inner();
    /// ```
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying value in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying value in this cursor.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying value as it may corrupt this cursor's position.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::io::Cursor;
    ///
    /// let mut buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the current position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncSeekExt, Cursor, SeekFrom};
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.seek(SeekFrom::Current(2)).await?;
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.seek(SeekFrom::Current(-1)).await?;
    /// assert_eq!(buff.position(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.set_position(4);
    /// assert_eq!(buff.position(), 4);
    /// ```
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T> AsyncSeek for Cursor<T>
where
    T: AsRef<[u8]> + Unpin,
{
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<Result<u64>> {
        let (base_pos, offset) = match pos {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Poll::Ready(Ok(n));
            }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u64)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as u64)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Poll::Ready(Ok(self.pos))
            }
            None => Poll::Ready(Err(Error::InvalidInput)),
        }
    }
}

impl<T: AsRef<[u8]> + Unpin> AsyncRead for Cursor<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        self.poll_read(cx, buf) // { io::Read::read(&mut self.inner, buf) }
    }
}

impl<T> AsyncBufRead for Cursor<T>
where
    T: AsRef<[u8]> + Unpin,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.poll_fill_buf(cx)
        // Poll::Ready(io::BufRead::fill_buf(&mut self.get_mut().inner))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        self.consume(amt)
        // io::BufRead::consume(&mut self.inner, amt)
    }
}

macro_rules! delegate_async_write {
    () => {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            self.poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            self.poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            self.poll_flush(cx)
        }
    };
}

impl AsyncWrite for Cursor<&mut [u8]> {
    delegate_async_write!();
}

impl AsyncWrite for Cursor<&mut Vec<u8>> {
    delegate_async_write!();
}

impl AsyncWrite for Cursor<Vec<u8>> {
    delegate_async_write!();
}

impl AsyncWrite for Cursor<Box<[u8]>> {
    delegate_async_write!();
}
