use crate::io::{self, AsyncRead, AsyncSeek, AsyncWrite};
use crate::Runtime;
use core::pin::Pin;
use core::task::{Context, Poll};
use nom_uri::{ToUri, Uri};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ResourceID {
    Sys(u8),
    InputGpio(u8),
    OutputGpio(u8),
    PWM(u8),
    Channel(u8),
    Serial(u8),
    Timer(u8),
    Virtual(u8),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ResourceError {
    NonReadingResource,
    NonWritingResource,
    Utf8Error(core::str::Utf8Error),
    FloatError(core::num::ParseFloatError),
    BusError,
    NotFound,
    /// The resource is ill configured for the desired task
    ConfigurationError,
    WriteError,
}

/// Inspired by the async io traits of the futures trait
pub trait Resource: Sync + ToUri {
    fn poll_read(&mut self, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<Result<usize, io::Error>>;
    fn poll_write(&mut self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>>;
    fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>>;
    fn poll_close(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>>;
    fn poll_seek(
        &mut self,
        cx: &mut Context<'_>,
        pos: io::SeekFrom,
    ) -> Poll<Result<u64, io::Error>>;
}

// impl Resource for ResourceID {}
impl Unpin for ResourceID {}
impl AsyncRead for ResourceID {
    fn poll_read(
        self: Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        Runtime::get()
            .get_resource_object(&*self)
            .poll_read(cx, buf)
    }
}
impl AsyncWrite for ResourceID {
    fn poll_write(
        self: Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Runtime::get()
            .get_resource_object(&*self)
            .poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut ResourceID>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Runtime::get().get_resource_object(&*self).poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut ResourceID>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Runtime::get().get_resource_object(&*self).poll_close(cx)
    }
}
impl AsyncSeek for ResourceID {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: io::SeekFrom,
    ) -> Poll<Result<u64, io::Error>> {
        Runtime::get()
            .get_resource_object(&*self)
            .poll_seek(cx, pos)
    }
}
impl ToUri for ResourceID {
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> Uri<'uri> {
        Runtime::get().get_resource_object(self).to_uri(buffer)
    }
}

impl From<core::num::ParseFloatError> for ResourceError {
    fn from(error: core::num::ParseFloatError) -> Self {
        ResourceError::FloatError(error)
    }
}
impl From<core::str::Utf8Error> for ResourceError {
    fn from(error: core::str::Utf8Error) -> Self {
        ResourceError::Utf8Error(error)
    }
}

pub struct ByteWriter<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
}
pub struct StrWriter<'a> {
    byte_writer: ByteWriter<'a>,
}
impl<'a> ByteWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }
    pub fn buffer(self) -> &'a mut [u8] {
        let (o, _) = self.buffer.split_at_mut(self.cursor);
        o
    }
    pub fn written(&self) -> usize {
        self.cursor
    }
}
impl<'a> StrWriter<'a> {
    pub fn new(buffer: &'a mut str) -> Self {
        Self {
            byte_writer: ByteWriter::new(unsafe { buffer.as_bytes_mut() }),
        }
    }
    pub fn buffer(self) -> Result<&'a mut str, core::str::Utf8Error> {
        core::str::from_utf8_mut(self.byte_writer.buffer())
    }
    pub fn written(&self) -> usize {
        self.byte_writer.written()
    }
}
impl<'a> core::fmt::Write for ByteWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        if self.buffer.len() - self.cursor < s.len() {
            log::error!("fmt::Write: Buffer to small");
            Err(core::fmt::Error)
        } else {
            for i in 0..s.len() {
                self.buffer[self.cursor] = s[i];
                self.cursor += 1;
            }
            Ok(())
        }
    }
}
impl<'a> core::fmt::Write for StrWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.byte_writer.write_str(s)
    }
}
impl<'buf> From<&'buf mut [u8]> for ByteWriter<'buf> {
    fn from(buffer: &'buf mut [u8]) -> Self {
        Self::new(buffer)
    }
}
impl<'buf> From<&'buf mut str> for ByteWriter<'buf> {
    fn from(buffer: &'buf mut str) -> Self {
        Self::new(unsafe { buffer.as_bytes_mut() })
    }
}
impl<'buf> From<&'buf mut str> for StrWriter<'buf> {
    fn from(buffer: &'buf mut str) -> Self {
        Self::new(buffer)
    }
}
