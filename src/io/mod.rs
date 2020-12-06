//! Asynchronous I/O
//!
//! Ported from `futures::io`
//!
//! This crate contains the `AsyncRead`, `AsyncWrite`, `AsyncSeek`, and
//! `AsyncBufRead` traits, the asynchronous analogs to
//! `std::io::{Read, Write, Seek, BufRead}`. The primary difference is
//! that these traits integrate with the asynchronous task system.
//!
//! All items of this library are only available when the `std` feature of this
//! library is activated, and it is activated by default.

#![warn(missing_docs, missing_debug_implementations, unreachable_pub)]
// It cannot be included in the published code because this lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![warn(clippy::all)]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![doc(html_root_url = "https://docs.rs/futures-io/0.3.5")]

#[cfg(all(feature = "read-initializer", not(feature = "unstable")))]
compile_error!("The `read-initializer` feature requires the `unstable` feature as an explicit opt-in to unstable features");

/// Extracts the successful type of a `Poll<T>`.
///
/// This macro bakes in propagation of `Pending` signals by returning early.
#[macro_use]
macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            core::task::Poll::Ready(t) => t,
            core::task::Poll::Pending => return core::task::Poll::Pending,
        }
    };
}

macro_rules! delegate_access_inner {
    ($field:ident, $inner:ty, ($($ind:tt)*)) => {
        /// Acquires a reference to the underlying sink or stream that this combinator is
        /// pulling from.
        pub fn get_ref(&self) -> &$inner {
            (&self.$field) $($ind get_ref())*
        }

        /// Acquires a mutable reference to the underlying sink or stream that this
        /// combinator is pulling from.
        ///
        /// Note that care must be taken to avoid tampering with the state of the
        /// sink or stream which may otherwise confuse this combinator.
        pub fn get_mut(&mut self) -> &mut $inner {
            (&mut self.$field) $($ind get_mut())*
        }

        /// Acquires a pinned mutable reference to the underlying sink or stream that this
        /// combinator is pulling from.
        ///
        /// Note that care must be taken to avoid tampering with the state of the
        /// sink or stream which may otherwise confuse this combinator.
        pub fn get_pin_mut(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut $inner> {
            self.project().$field $($ind get_pin_mut())*
        }

        /// Consumes this combinator, returning the underlying sink or stream.
        ///
        /// Note that this may discard intermediate state of this combinator, so
        /// care should be taken to avoid losing resources when this is called.
        pub fn into_inner(self) -> $inner {
            self.$field $($ind into_inner())*
        }
    }
}

macro_rules! delegate_async_buf_read {
    ($field:ident) => {
        fn poll_fill_buf(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Result<&[u8]>> {
            self.project().$field.poll_fill_buf(cx)
        }

        fn consume(self: core::pin::Pin<&mut Self>, amt: usize) {
            self.project().$field.consume(amt)
        }
    };
}
macro_rules! delegate_async_write {
    ($field:ident) => {
        fn poll_write(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
            buf: &[u8],
        ) -> core::task::Poll<Result<usize>> {
            self.project().$field.poll_write(cx, buf)
        }
        fn poll_flush(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Result<()>> {
            self.project().$field.poll_flush(cx)
        }
        fn poll_close(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Result<()>> {
            self.project().$field.poll_close(cx)
        }
    };
}

macro_rules! delegate_async_read {
    ($field:ident) => {
        #[cfg(feature = "read-initializer")]
        unsafe fn initializer(&self) -> $crate::io::Initializer {
            self.$field.initializer()
        }

        fn poll_read(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
            buf: &mut [u8],
        ) -> core::task::Poll<Result<usize>> {
            self.project().$field.poll_read(cx, buf)
        }
    };
}

mod buf_reader;
pub use self::buf_reader::BufReader;

mod buf_writer;
pub use self::buf_writer::BufWriter;

mod chain;
pub use self::chain::Chain;

mod close;
pub use self::close::Close;

mod copy;
pub use self::copy::{copy, Copy};

mod copy_buf;
pub use self::copy_buf::{copy_buf, CopyBuf};

// TODO: implement?
// mod cursor;
// pub use self::cursor::Cursor;

mod empty;
pub use self::empty::{empty, Empty};

mod flush;
pub use self::flush::Flush;

// TODO: implement?
// mod into_sink;
// pub use self::into_sink::IntoSink;

mod lines;
pub use self::lines::Lines;

mod read;
pub use self::read::Read;

mod read_exact;
pub use self::read_exact::ReadExact;

mod read_line;
pub use self::read_line::ReadLine;

mod read_to_end;
pub use self::read_to_end::ReadToEnd;

mod read_to_string;
pub use self::read_to_string::ReadToString;

mod read_until;
pub use self::read_until::ReadUntil;

mod repeat;
pub use self::repeat::{repeat, Repeat};

mod seek;
pub use self::seek::Seek;

mod sink;
pub use self::sink::{sink, Sink};

// TODO: implement?
// mod split;
// pub use self::split::{ReadHalf, ReuniteError, WriteHalf};

mod take;
pub use self::take::Take;

mod window;
pub use self::window::Window;

mod write;
pub use self::write::Write;

mod write_all;
pub use self::write_all::WriteAll;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Error {
    /// An entity was not found, often a file.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// initialized.
    AddrNotAvailable,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// A parameter was incorrect.
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    WriteZero,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,
    /// Any I/O error not part of this list.
    ///
    /// Errors that are `Other` now may move to a different or a new
    /// [`ErrorKind`] variant in the future. It is not recommended to match
    /// an error against `Other` and to expect any additional characteristics,
    /// e.g., a specific [`Error::raw_os_error`] return value.
    Other,
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,
}
pub type Result<T> = core::result::Result<T, Error>;

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
///
/// [`Seek`]: trait.Seek.html
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),

    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),

    /// Sets the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current(i64),
}

/// Read bytes asynchronously.
///
/// This trait is analogous to the `std::io::Read` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_read`
/// method, unlike `Read::read`, will automatically queue the current task
/// for wakeup and return if data is not yet available, rather than blocking
/// the calling thread.
pub trait AsyncRead {
    /// Attempt to read from the `AsyncRead` into `buf`.
    ///
    /// On success, returns `Poll::Ready(Ok(num_bytes_read))`.
    ///
    /// If no data is available for reading, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// readable or is closed.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<Result<usize>>;
}

/// Write bytes asynchronously.
///
/// This trait is analogous to the `std::io::Write` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_write`
/// method, unlike `Write::write`, will automatically queue the current task
/// for wakeup and return if the writer cannot take more data, rather than blocking
/// the calling thread.
pub trait AsyncWrite {
    /// Attempt to write bytes from `buf` into the object.
    ///
    /// On success, returns `Poll::Ready(Ok(num_bytes_written))`.
    ///
    /// If the object is not ready for writing, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// writable or is closed.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    ///
    /// `poll_write` must try to make progress by flushing the underlying object if
    /// that is the only way the underlying object can become writable again.
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>>;

    /// Attempt to flush the object, ensuring that any buffered data reach
    /// their destination.
    ///
    /// On success, returns `Poll::Ready(Ok(()))`.
    ///
    /// If flushing cannot immediately complete, this method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object can make
    /// progress towards flushing.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    ///
    /// It only makes sense to do anything here if you actually buffer data.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>;

    /// Attempt to close the object.
    ///
    /// On success, returns `Poll::Ready(Ok(()))`.
    ///
    /// If closing cannot immediately complete, this function returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object can make
    /// progress towards closing.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>;
}

/// Seek bytes asynchronously.
///
/// This trait is analogous to the `std::io::Seek` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_seek`
/// method, unlike `Seek::seek`, will automatically queue the current task
/// for wakeup and return if data is not yet available, rather than blocking
/// the calling thread.
pub trait AsyncSeek {
    /// Attempt to seek to an offset, in bytes, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but behavior is defined
    /// by the implementation.
    ///
    /// If the seek operation completed successfully,
    /// this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    ///
    /// # Errors
    ///
    /// Seeking to a negative offset is considered an error.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>>;
}

/// Read bytes asynchronously.
///
/// This trait is analogous to the `std::io::BufRead` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_fill_buf`
/// method, unlike `BufRead::fill_buf`, will automatically queue the current task
/// for wakeup and return if data is not yet available, rather than blocking
/// the calling thread.
pub trait AsyncBufRead: AsyncRead {
    /// Attempt to return the contents of the internal buffer, filling it with more data
    /// from the inner reader if it is empty.
    ///
    /// On success, returns `Poll::Ready(Ok(buf))`.
    ///
    /// If no data is available for reading, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// readable or is closed.
    ///
    /// This function is a lower-level call. It needs to be paired with the
    /// [`consume`] method to function properly. When calling this
    /// method, none of the contents will be "read" in the sense that later
    /// calling [`poll_read`] may return the same contents. As such, [`consume`] must
    /// be called with the number of bytes that are consumed from this buffer to
    /// ensure that the bytes are never returned twice.
    ///
    /// [`poll_read`]: AsyncRead::poll_read
    /// [`consume`]: AsyncBufRead::consume
    ///
    /// An empty buffer returned indicates that the stream has reached EOF.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>>;

    /// Tells this buffer that `amt` bytes have been consumed from the buffer,
    /// so they should no longer be returned in calls to [`poll_read`].
    ///
    /// This function is a lower-level call. It needs to be paired with the
    /// [`poll_fill_buf`] method to function properly. This function does
    /// not perform any I/O, it simply informs this object that some amount of
    /// its buffer, returned from [`poll_fill_buf`], has been consumed and should
    /// no longer be returned. As such, this function may do odd things if
    /// [`poll_fill_buf`] isn't called before calling it.
    ///
    /// The `amt` must be `<=` the number of bytes in the buffer returned by
    /// [`poll_fill_buf`].
    ///
    /// [`poll_read`]: AsyncRead::poll_read
    /// [`poll_fill_buf`]: AsyncBufRead::poll_fill_buf
    fn consume(self: Pin<&mut Self>, amt: usize);
}

macro_rules! deref_async_read {
    () => {
        #[cfg(feature = "read-initializer")]
        unsafe fn initializer(&self) -> Initializer {
            (**self).initializer()
        }

        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut **self).poll_read(cx, buf)
        }
    };
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
    deref_async_read!();
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    deref_async_read!();
}

impl<P> AsyncRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncRead,
{
    #[cfg(feature = "read-initializer")]
    unsafe fn initializer(&self) -> Initializer {
        (**self).initializer()
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_read(cx, buf)
    }
}

impl AsyncRead for &[u8] {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Poll::Ready(Ok(amt))
    }
}

macro_rules! deref_async_write {
    () => {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            Pin::new(&mut **self).poll_flush(cx)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            Pin::new(&mut **self).poll_close(cx)
        }
    };
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for Box<T> {
    deref_async_write!();
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    deref_async_write!();
}

impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.get_mut().as_mut().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.get_mut().as_mut().poll_close(cx)
    }
}

impl AsyncWrite for Vec<u8> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        self.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.poll_flush(cx)
    }
}

macro_rules! deref_async_seek {
    () => {
        fn poll_seek(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: SeekFrom,
        ) -> Poll<Result<u64>> {
            Pin::new(&mut **self).poll_seek(cx, pos)
        }
    };
}

impl<T: ?Sized + AsyncSeek + Unpin> AsyncSeek for Box<T> {
    deref_async_seek!();
}

impl<T: ?Sized + AsyncSeek + Unpin> AsyncSeek for &mut T {
    deref_async_seek!();
}

impl<P> AsyncSeek for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncSeek,
{
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>> {
        self.get_mut().as_mut().poll_seek(cx, pos)
    }
}

macro_rules! deref_async_buf_read {
    () => {
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
            Pin::new(&mut **self.get_mut()).poll_fill_buf(cx)
        }

        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            Pin::new(&mut **self).consume(amt)
        }
    };
}

impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for Box<T> {
    deref_async_buf_read!();
}

impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for &mut T {
    deref_async_buf_read!();
}

impl<P> AsyncBufRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncBufRead,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.get_mut().as_mut().poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.get_mut().as_mut().consume(amt)
    }
}

// used by `BufReader` and `BufWriter`
// https://github.com/rust-lang/rust/blob/master/src/libstd/sys_common/io.rs#L1
const DEFAULT_BUF_SIZE: usize = 255;

/// An extension trait which adds utility methods to `AsyncRead` types.
pub trait AsyncReadExt: AsyncRead {
    /// Creates an adaptor which will chain this stream with another.
    ///
    /// The returned `AsyncRead` instance will first read all bytes from this object
    /// until EOF is encountered. Afterwards the output is equivalent to the
    /// output of `next`.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let reader1 = Cursor::new([1, 2, 3, 4]);
    /// let reader2 = Cursor::new([5, 6, 7, 8]);
    ///
    /// let mut reader = reader1.chain(reader2);
    /// let mut buffer = Vec::new();
    ///
    /// // read the value into a Vec.
    /// reader.read_to_end(&mut buffer).await?;
    /// assert_eq!(buffer, [1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn chain<R>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
        R: AsyncRead,
    {
        Chain::new(self, next)
    }

    /// Tries to read some bytes directly into the given `buf` in asynchronous
    /// manner, returning a future type.
    ///
    /// The returned future will resolve to the number of bytes read once the read
    /// operation is completed.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let mut reader = Cursor::new([1, 2, 3, 4]);
    /// let mut output = [0u8; 5];
    ///
    /// let bytes = reader.read(&mut output[..]).await?;
    ///
    /// // This is only guaranteed to be 4 because `&[u8]` is a synchronous
    /// // reader. In a real system you could get anywhere from 1 to
    /// // `output.len()` bytes in a single read.
    /// assert_eq!(bytes, 4);
    /// assert_eq!(output, [1, 2, 3, 4, 0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        Read::new(self, buf)
    }

    /// Creates a future which will read exactly enough bytes to fill `buf`,
    /// returning an error if end of file (EOF) is hit sooner.
    ///
    /// The returned future will resolve once the read operation is completed.
    ///
    /// In the case of an error the buffer and the object will be discarded, with
    /// the error yielded.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let mut reader = Cursor::new([1, 2, 3, 4]);
    /// let mut output = [0u8; 4];
    ///
    /// reader.read_exact(&mut output).await?;
    ///
    /// assert_eq!(output, [1, 2, 3, 4]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    ///
    /// ## EOF is hit before `buf` is filled
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{self, AsyncReadExt, Cursor};
    ///
    /// let mut reader = Cursor::new([1, 2, 3, 4]);
    /// let mut output = [0u8; 5];
    ///
    /// let result = reader.read_exact(&mut output).await;
    ///
    /// assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    /// # });
    /// ```
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadExact<'a, Self>
    where
        Self: Unpin,
    {
        ReadExact::new(self, buf)
    }

    /// Creates a future which will read all the bytes from this `AsyncRead`.
    ///
    /// On success the total number of bytes read is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let mut reader = Cursor::new([1, 2, 3, 4]);
    /// let mut output = Vec::with_capacity(4);
    ///
    /// let bytes = reader.read_to_end(&mut output).await?;
    ///
    /// assert_eq!(bytes, 4);
    /// assert_eq!(output, vec![1, 2, 3, 4]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn read_to_end<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> ReadToEnd<'a, Self>
    where
        Self: Unpin,
    {
        ReadToEnd::new(self, buf)
    }

    /// Creates a future which will read all the bytes from this `AsyncRead`.
    ///
    /// On success the total number of bytes read is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let mut reader = Cursor::new(&b"1234"[..]);
    /// let mut buffer = String::with_capacity(4);
    ///
    /// let bytes = reader.read_to_string(&mut buffer).await?;
    ///
    /// assert_eq!(bytes, 4);
    /// assert_eq!(buffer, String::from("1234"));
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn read_to_string<'a>(&'a mut self, buf: &'a mut String) -> ReadToString<'a, Self>
    where
        Self: Unpin,
    {
        ReadToString::new(self, buf)
    }

    // /// Helper method for splitting this read/write object into two halves.
    // ///
    // /// The two halves returned implement the `AsyncRead` and `AsyncWrite`
    // /// traits, respectively.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// # futures::executor::block_on(async {
    // /// use futures::io::{self, AsyncReadExt, Cursor};
    // ///
    // /// // Note that for `Cursor` the read and write halves share a single
    // /// // seek position. This may or may not be true for other types that
    // /// // implement both `AsyncRead` and `AsyncWrite`.
    // ///
    // /// let reader = Cursor::new([1, 2, 3, 4]);
    // /// let mut buffer = Cursor::new(vec![0, 0, 0, 0, 5, 6, 7, 8]);
    // /// let mut writer = Cursor::new(vec![0u8; 5]);
    // ///
    // /// {
    // ///     let (buffer_reader, mut buffer_writer) = (&mut buffer).split();
    // ///     io::copy(reader, &mut buffer_writer).await?;
    // ///     io::copy(buffer_reader, &mut writer).await?;
    // /// }
    // ///
    // /// assert_eq!(buffer.into_inner(), [1, 2, 3, 4, 5, 6, 7, 8]);
    // /// assert_eq!(writer.into_inner(), [5, 6, 7, 8, 0]);
    // /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    // /// ```
    // fn split(self) -> (ReadHalf<Self>, WriteHalf<Self>)
    // where
    //     Self: AsyncWrite + Sized,
    // {
    //     split::split(self)
    // }

    /// Creates an AsyncRead adapter which will read at most `limit` bytes
    /// from the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncReadExt, Cursor};
    ///
    /// let reader = Cursor::new(&b"12345678"[..]);
    /// let mut buffer = [0; 5];
    ///
    /// let mut take = reader.take(4);
    /// let n = take.read(&mut buffer).await?;
    ///
    /// assert_eq!(n, 4);
    /// assert_eq!(&buffer, b"1234\0");
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, limit)
    }

    /// Wraps an [`AsyncRead`] in a compatibility wrapper that allows it to be
    /// used as a futures 0.1 / tokio-io 0.1 `AsyncRead`. If the wrapped type
    /// implements [`AsyncWrite`] as well, the result will also implement the
    /// futures 0.1 / tokio 0.1 `AsyncWrite` trait.
    ///
    /// Requires the `io-compat` feature to enable.
    #[cfg(feature = "io-compat")]
    fn compat(self) -> Compat<Self>
    where
        Self: Sized + Unpin,
    {
        Compat::new(self)
    }
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}

/// An extension trait which adds utility methods to `AsyncWrite` types.
pub trait AsyncWriteExt: AsyncWrite {
    /// Creates a future which will entirely flush this `AsyncWrite`.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AllowStdIo, AsyncWriteExt};
    /// use std::io::{BufWriter, Cursor};
    ///
    /// let mut output = vec![0u8; 5];
    ///
    /// {
    ///     let writer = Cursor::new(&mut output);
    ///     let mut buffered = AllowStdIo::new(BufWriter::new(writer));
    ///     buffered.write_all(&[1, 2]).await?;
    ///     buffered.write_all(&[3, 4]).await?;
    ///     buffered.flush().await?;
    /// }
    ///
    /// assert_eq!(output, [1, 2, 3, 4, 0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Unpin,
    {
        Flush::new(self)
    }

    /// Creates a future which will entirely close this `AsyncWrite`.
    fn close(&mut self) -> Close<'_, Self>
    where
        Self: Unpin,
    {
        Close::new(self)
    }

    /// Creates a future which will write bytes from `buf` into the object.
    ///
    /// The returned future will resolve to the number of bytes written once the write
    /// operation is completed.
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Write<'a, Self>
    where
        Self: Unpin,
    {
        Write::new(self, buf)
    }

    /// Write data into this object.
    ///
    /// Creates a future that will write the entire contents of the buffer `buf` into
    /// this `AsyncWrite`.
    ///
    /// The returned future will not complete until all the data has been written.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncWriteExt, Cursor};
    ///
    /// let mut writer = Cursor::new(vec![0u8; 5]);
    ///
    /// writer.write_all(&[1, 2, 3, 4]).await?;
    ///
    /// assert_eq!(writer.into_inner(), [1, 2, 3, 4, 0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> WriteAll<'a, Self>
    where
        Self: Unpin,
    {
        WriteAll::new(self, buf)
    }

    // / Allow using an [`AsyncWrite`] as a [`Sink`](futures_sink::Sink)`<Item: AsRef<[u8]>>`.
    // /
    // / This adapter produces a sink that will write each value passed to it
    // / into the underlying writer.
    // /
    // / Note that this function consumes the given writer, returning a wrapped
    // / version.
    // /
    // / # Examples
    // /
    // / ```
    // / # futures::executor::block_on(async {
    // / use futures::io::AsyncWriteExt;
    // / use futures::stream::{self, StreamExt};
    // /
    // / let stream = stream::iter(vec![Ok([1, 2, 3]), Ok([4, 5, 6])]);
    // /
    // / let mut writer = vec![];
    // /
    // / stream.forward((&mut writer).into_sink()).await?;
    // /
    // / assert_eq!(writer, vec![1, 2, 3, 4, 5, 6]);
    // / # Ok::<(), Box<dyn std::error::Error>>(())
    // / # })?;
    // / # Ok::<(), Box<dyn std::error::Error>>(())
    // / ```
    // TODO: implement?
    // fn into_sink<Item: AsRef<[u8]>>(self) -> IntoSink<Self, Item>
    // where
    //     Self: Sized,
    // {
    //     IntoSink::new(self)
    // }
}

impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}

/// An extension trait which adds utility methods to `AsyncSeek` types.
pub trait AsyncSeekExt: AsyncSeek {
    /// Creates a future which will seek an IO object, and then yield the
    /// new position in the object and the object itself.
    ///
    /// In the case of an error the buffer and the object will be discarded, with
    /// the error yielded.
    fn seek(&mut self, pos: SeekFrom) -> Seek<'_, Self>
    where
        Self: Unpin,
    {
        Seek::new(self, pos)
    }
}

impl<S: AsyncSeek + ?Sized> AsyncSeekExt for S {}

/// An extension trait which adds utility methods to `AsyncBufRead` types.
pub trait AsyncBufReadExt: AsyncBufRead {
    /// Creates a future which will read all the bytes associated with this I/O
    /// object into `buf` until the delimiter `byte` or EOF is reached.
    /// This method is the async equivalent to [`BufRead::read_until`](std::io::BufRead::read_until).
    ///
    /// This function will read bytes from the underlying stream until the
    /// delimiter or EOF is found. Once found, all bytes up to, and including,
    /// the delimiter (if found) will be appended to `buf`.
    ///
    /// The returned future will resolve to the number of bytes read once the read
    /// operation is completed.
    ///
    /// In the case of an error the buffer and the object will be discarded, with
    /// the error yielded.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncBufReadExt, Cursor};
    ///
    /// let mut cursor = Cursor::new(b"lorem-ipsum");
    /// let mut buf = vec![];
    ///
    /// // cursor is at 'l'
    /// let num_bytes = cursor.read_until(b'-', &mut buf).await?;
    /// assert_eq!(num_bytes, 6);
    /// assert_eq!(buf, b"lorem-");
    /// buf.clear();
    ///
    /// // cursor is at 'i'
    /// let num_bytes = cursor.read_until(b'-', &mut buf).await?;
    /// assert_eq!(num_bytes, 5);
    /// assert_eq!(buf, b"ipsum");
    /// buf.clear();
    ///
    /// // cursor is at EOF
    /// let num_bytes = cursor.read_until(b'-', &mut buf).await?;
    /// assert_eq!(num_bytes, 0);
    /// assert_eq!(buf, b"");
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn read_until<'a>(&'a mut self, byte: u8, buf: &'a mut Vec<u8>) -> ReadUntil<'a, Self>
    where
        Self: Unpin,
    {
        ReadUntil::new(self, byte, buf)
    }

    /// Creates a future which will read all the bytes associated with this I/O
    /// object into `buf` until a newline (the 0xA byte) or EOF is reached,
    /// This method is the async equivalent to [`BufRead::read_line`](std::io::BufRead::read_line).
    ///
    /// This function will read bytes from the underlying stream until the
    /// newline delimiter (the 0xA byte) or EOF is found. Once found, all bytes
    /// up to, and including, the delimiter (if found) will be appended to
    /// `buf`.
    ///
    /// The returned future will resolve to the number of bytes read once the read
    /// operation is completed.
    ///
    /// In the case of an error the buffer and the object will be discarded, with
    /// the error yielded.
    ///
    /// # Errors
    ///
    /// This function has the same error semantics as [`read_until`] and will
    /// also return an error if the read bytes are not valid UTF-8. If an I/O
    /// error is encountered then `buf` may contain some bytes already read in
    /// the event that all data read so far was valid UTF-8.
    ///
    /// [`read_until`]: AsyncBufReadExt::read_until
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncBufReadExt, Cursor};
    ///
    /// let mut cursor = Cursor::new(b"foo\nbar");
    /// let mut buf = String::new();
    ///
    /// // cursor is at 'f'
    /// let num_bytes = cursor.read_line(&mut buf).await?;
    /// assert_eq!(num_bytes, 4);
    /// assert_eq!(buf, "foo\n");
    /// buf.clear();
    ///
    /// // cursor is at 'b'
    /// let num_bytes = cursor.read_line(&mut buf).await?;
    /// assert_eq!(num_bytes, 3);
    /// assert_eq!(buf, "bar");
    /// buf.clear();
    ///
    /// // cursor is at EOF
    /// let num_bytes = cursor.read_line(&mut buf).await?;
    /// assert_eq!(num_bytes, 0);
    /// assert_eq!(buf, "");
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn read_line<'a>(&'a mut self, buf: &'a mut String) -> ReadLine<'a, Self>
    where
        Self: Unpin,
    {
        ReadLine::new(self, buf)
    }

    /// Returns a stream over the lines of this reader.
    /// This method is the async equivalent to [`BufRead::lines`](std::io::BufRead::lines).
    ///
    /// The stream returned from this function will yield instances of
    /// [`io::Result`]`<`[`String`]`>`. Each string returned will *not* have a newline
    /// byte (the 0xA byte) or CRLF (0xD, 0xA bytes) at the end.
    ///
    /// [`io::Result`]: std::io::Result
    /// [`String`]: String
    ///
    /// # Errors
    ///
    /// Each line of the stream has the same error semantics as [`AsyncBufReadExt::read_line`].
    ///
    /// [`AsyncBufReadExt::read_line`]: AsyncBufReadExt::read_line
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::io::{AsyncBufReadExt, Cursor};
    /// use futures::stream::StreamExt;
    ///
    /// let cursor = Cursor::new(b"lorem\nipsum\r\ndolor");
    ///
    /// let mut lines_stream = cursor.lines().map(|l| l.unwrap());
    /// assert_eq!(lines_stream.next().await, Some(String::from("lorem")));
    /// assert_eq!(lines_stream.next().await, Some(String::from("ipsum")));
    /// assert_eq!(lines_stream.next().await, Some(String::from("dolor")));
    /// assert_eq!(lines_stream.next().await, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
    /// ```
    fn lines(self) -> Lines<Self>
    where
        Self: Sized,
    {
        Lines::new(self)
    }
}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadExt for R {}
