#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourceID(pub u8);

#[non_exhaustive]
pub enum ResourceError {
    NonReadingResource,
    NonWritingResource,
    Utf8Error(core::str::Utf8Error),
    FloatError(core::num::ParseFloatError),
}

#[allow(unused)]
pub trait Resource {
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
        ResourceError::NonReadingResource.into()
    }
    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
        ResourceError::NonWritingResource.into()
    }
    fn seek(&mut self, pos: usize) -> nb::Result<(), ResourceError>;
    fn flush(&mut self) -> nb::Result<(), ResourceError>;
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
impl<T> From<ResourceError> for nb::Result<T, ResourceError> {
    fn from(error: ResourceError) -> Self {
        Err(nb::Error::Other(error))
    }
}

pub struct ByteWriter<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
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
impl<'a> core::fmt::Write for ByteWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        if self.buffer.len() - self.cursor < s.len() {
            log::error!("Buffer to small");
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