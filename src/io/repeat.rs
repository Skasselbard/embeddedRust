use super::{AsyncRead, Result};
use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
#[cfg(feature = "read-initializer")]
use futures_io::Initializer;

/// Reader for the [`repeat()`] function.
#[must_use = "readers do nothing unless polled"]
pub struct Repeat {
    byte: u8,
}

/// Creates an instance of a reader that infinitely repeats one byte.
///
/// All reads from this reader will succeed by filling the specified buffer with
/// the given byte.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::io::{self, AsyncReadExt};
///
/// let mut buffer = [0; 3];
/// let mut reader = io::repeat(0b101);
/// reader.read_exact(&mut buffer).await.unwrap();
/// assert_eq!(buffer, [0b101, 0b101, 0b101]);
/// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
/// ```
pub fn repeat(byte: u8) -> Repeat {
    Repeat { byte }
}

impl AsyncRead for Repeat {
    #[inline]
    fn poll_read(self: Pin<&mut Self>, _: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        for slot in &mut *buf {
            *slot = self.byte;
        }
        Poll::Ready(Ok(buf.len()))
    }
}

impl fmt::Debug for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Repeat { .. }")
    }
}
