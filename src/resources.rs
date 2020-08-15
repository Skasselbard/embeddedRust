use super::ComponentConfiguration;
use crate::Runtime;
use core::task::{Context, Poll};
use futures::StreamExt;
use nom_uri::Uri;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ResourceID {
    Sys(u8),
    InputGpio(u8),
    OutputGpio(u8),
    PWM(u8),
    Channel(u8),
    Serial(u8),
    Timer(u8),
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

pub trait RegisterComponent {
    fn register_component(self, configuration: ComponentConfiguration);
}

#[allow(unused)]
pub trait Resource: Sync {
    fn read_next(&mut self, context: &mut Context) -> Poll<Option<u8>> {
        Poll::Ready(None)
    }
    fn write_next(&mut self, context: &mut Context, byte: u8) -> Poll<Result<(), ResourceError>> {
        Poll::Ready(Err(ResourceError::NonWritingResource))
    }
    fn seek(&mut self, context: &mut Context, pos: usize) -> Poll<Result<(), ResourceError>>;
}

// #[allow(unused)]
// impl ResourceID {
//     pub fn read_stream(&mut self) -> impl StreamExt<Item = u8> {
//         use futures::stream::poll_fn;
//         let id = *self;
//         poll_fn(move |cx| Runtime::get().get_resource_object(&id).read_next(cx))
//     }
//     pub async fn write(
//         &mut self,
//         mut stream: impl StreamExt<Item = u8> + Unpin,
//     ) -> Result<(), ResourceError> {
//         use futures::future::poll_fn;

//         let res = Runtime::get().get_resource_object(self);
//         while let Some(byte) = stream.next().await {
//             poll_fn(|cx| (res.write_next(cx, byte))).await?
//         }
//         Ok(())
//     }
//     pub async fn seek(&mut self, pos: usize) -> Result<(), ResourceError> {
//         use futures::future::poll_fn;
//         poll_fn(|cx| Runtime::get().get_resource_object(self).seek(cx, pos)).await
//     }
//     pub fn to_uri<'uri>(&self, buffer: &'uri mut str) -> Uri<'uri> {
//         Runtime::get().get_resource_object(self).to_uri(buffer)
//     }
// }

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
