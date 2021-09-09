pub mod gpio;
pub mod path;
pub mod pwm;
pub mod serial;
pub mod sys;

use crate::schemes::{Analog, Bus, Digital, Event, Memory, Percent, Scheme, Sys};
use crate::{
    io::{self, AsyncRead, AsyncSeek, AsyncWrite},
    Runtime,
};
use core::task::{Context, Poll};
use nom_uri::Uri;
use path::{IndexedPath, RawPath, ResourceMode};
use pwm::PWMMode;

pub use gpio::{InputPin, OutputPin, Pin};
pub use pwm::PWMPin;
pub use serial::Serial;
pub use sys::SysResource;

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
    ConversionError,
    UriParseError,
    Unresolvable,
    ParseError,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct ResourceID {
    /// Determines the data format which the resource accepts and returns
    scheme: Scheme,
    /// Indes into a specific array
    index: IndexedPath,
    /// Resources function diferently in different modes
    mode: ResourceMode,
}

pub(crate) struct Resources {
    pub(crate) sys: &'static mut [&'static mut dyn Resource],
    pub(crate) input_pins: &'static mut [&'static mut dyn Resource],
    pub(crate) output_pins: &'static mut [&'static mut dyn Resource],
    pub(crate) pwm: &'static mut [&'static mut dyn Resource],
    pub(crate) channels: &'static mut [&'static mut dyn Resource],
    pub(crate) serials: &'static mut [&'static mut dyn Resource],
    pub(crate) timers: &'static mut [&'static mut dyn Resource],
}

impl Resources {
    pub fn new(
        sys: &'static mut [&'static mut dyn Resource],
        input_pins: &'static mut [&'static mut dyn Resource],
        output_pins: &'static mut [&'static mut dyn Resource],
        pwm: &'static mut [&'static mut dyn Resource],
        channels: &'static mut [&'static mut dyn Resource],
        serials: &'static mut [&'static mut dyn Resource],
        timers: &'static mut [&'static mut dyn Resource],
    ) -> Self {
        Self {
            sys,
            input_pins,
            output_pins,
            pwm,
            channels,
            serials,
            timers,
        }
    }
    fn get_resource_object(&'static mut self, id: &ResourceID) -> &mut dyn Resource {
        match id.get_index() {
            IndexedPath::Sys(index) => *self.sys.get_mut(index as usize).unwrap(),
            IndexedPath::InputGpio(index) => *self.input_pins.get_mut(index as usize).unwrap(),
            IndexedPath::OutputGpio(index) => *self.output_pins.get_mut(index as usize).unwrap(),
            IndexedPath::PWM(index) => *self.pwm.get_mut(index as usize).unwrap(),
            IndexedPath::ADCPin(index) => *self.channels.get_mut(index as usize).unwrap(),
            IndexedPath::Serial(index) => *self.serials.get_mut(index as usize).unwrap(),
            IndexedPath::Timer(index) => *self.timers.get_mut(index as usize).unwrap(),
        }
    }
    pub fn get_resource(&'static mut self, uri: &str) -> Result<ResourceID, ResourceError> {
        use core::convert::TryFrom;
        use core::str::FromStr;
        let parsed_uri = Uri::try_from(uri).or(Err(ResourceError::UriParseError))?;
        let (index, mode) = RawPath::from_str(parsed_uri.path())?.resolve()?;
        Ok(ResourceID::new(
            Scheme::from_str(parsed_uri.scheme()).map_err(|_| ResourceError::UriParseError)?,
            index,
            mode,
        ))
    }
    fn search_resource_array(
        &self,
        path: &RawPath,
        array: &[&mut dyn Resource],
    ) -> Result<u8, ResourceError> {
        for i in 0..array.len() {
            if path == &array[i].path() {
                return Ok(i as u8);
            }
        }
        Err(ResourceError::NotFound)
    }
    pub(crate) fn get_input_pin(pin: &Pin) -> Result<&mut dyn Resource, ResourceError> {
        let path = RawPath::Gpio(*pin);
        let resources = Runtime::get_resources();
        let index = resources.search_resource_array(&path, resources.input_pins)?;
        Ok(resources.input_pins[index as usize])
    }
}
#[derive(Debug)]
pub struct ResourceConfig<'rsrc, 'ctx> {
    pub context: &'rsrc Context<'ctx>,
    pub scheme: Scheme,
    pub mode: ResourceMode,
}

/// Inspired by the async io traits of the futures trait
pub trait Resource {
    fn poll_read(
        &mut self,
        config: &ResourceConfig,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>>;
    fn poll_write(&mut self, config: &ResourceConfig, buf: &[u8])
        -> Poll<Result<usize, io::Error>>;
    fn poll_flush(&mut self, config: &ResourceConfig) -> Poll<Result<(), io::Error>>;
    fn poll_close(&mut self, config: &ResourceConfig) -> Poll<Result<(), io::Error>>;
    fn poll_seek(
        &mut self,
        config: &ResourceConfig,
        pos: io::SeekFrom,
    ) -> Poll<Result<u64, io::Error>>;
    //TODO: try to get rid of handle event
    fn handle_event(&mut self);
    fn path(&self) -> RawPath;
}

impl ResourceID {
    pub(crate) fn new(scheme: Scheme, index: IndexedPath, mode: ResourceMode) -> Self {
        Self {
            scheme,
            index,
            mode,
        }
    }
    pub(crate) fn get_index(&self) -> IndexedPath {
        self.index
    }
    pub fn into_memory(self) -> Result<Memory, ResourceError> {
        unimplemented!()
    }
    pub fn into_bus(self) -> Result<Bus, ResourceError> {
        unimplemented!()
    }
    pub fn into_analog(self) -> Result<Analog, ResourceError> {
        match self.index {
            IndexedPath::PWM(_) => {
                if let ResourceMode::PWM(PWMMode::Default) = self.mode {
                    {
                        Ok(Analog::new(self.index, self.mode))
                    }
                } else {
                    Err(ResourceError::ConversionError)
                }
            }
            IndexedPath::ADCPin(_) => Ok(Analog::new(self.index, self.mode)),
            _ => Err(ResourceError::ConversionError),
        }
    }
    pub fn into_digital(self) -> Result<Digital, ResourceError> {
        match self.index {
            IndexedPath::InputGpio(_) | IndexedPath::OutputGpio(_) => {
                Ok(Digital::new(self.index, self.mode))
            }
            _ => Err(ResourceError::ConversionError),
        }
    }
    pub fn into_event(self) -> Result<Event, ResourceError> {
        match self.index {
            IndexedPath::InputGpio(_) => Ok(Event::new(self.index, self.mode)),
            //TODO: Timer/ Serial
            _ => Err(ResourceError::ConversionError),
        }
    }
    pub fn into_sys(self) -> Result<Sys, ResourceError> {
        unimplemented!()
    }
    pub fn into_percent(self) -> Result<Percent, ResourceError> {
        unimplemented!()
    }
}
impl Unpin for ResourceID {}
impl AsyncRead for ResourceID {
    fn poll_read(
        self: core::pin::Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        let config = ResourceConfig {
            context: cx,
            scheme: self.scheme,
            mode: self.mode,
        };
        Runtime::get_resources()
            .get_resource_object(&*self)
            .poll_read(&config, buf)
    }
}
impl AsyncWrite for ResourceID {
    fn poll_write(
        self: core::pin::Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let config = ResourceConfig {
            context: cx,
            scheme: self.scheme,
            mode: self.mode,
        };
        Runtime::get_resources()
            .get_resource_object(&*self)
            .poll_write(&config, buf)
    }
    fn poll_flush(
        self: core::pin::Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let config = ResourceConfig {
            context: cx,
            scheme: self.scheme,
            mode: self.mode,
        };
        Runtime::get_resources()
            .get_resource_object(&*self)
            .poll_flush(&config)
    }
    fn poll_close(
        self: core::pin::Pin<&mut ResourceID>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let config = ResourceConfig {
            context: cx,
            scheme: self.scheme,
            mode: self.mode,
        };
        Runtime::get_resources()
            .get_resource_object(&*self)
            .poll_close(&config)
    }
}
impl AsyncSeek for ResourceID {
    fn poll_seek(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: io::SeekFrom,
    ) -> Poll<Result<u64, io::Error>> {
        let config = ResourceConfig {
            context: cx,
            scheme: self.scheme,
            mode: self.mode,
        };
        Runtime::get_resources()
            .get_resource_object(&*self)
            .poll_seek(&config, pos)
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
