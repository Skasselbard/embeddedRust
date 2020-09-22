mod cortex_m;

#[cfg(feature = "stm32f1xx")]
mod stm32f1xx;
#[cfg(feature = "stm32f1xx")]
pub(crate) use self::cortex_m::init_heap;
#[cfg(feature = "stm32f1xx")]
use stm32f1xx as dev;

use crate::alloc::string::ToString;
use crate::io::{self, SeekFrom};
use crate::resources::StrWriter;
use crate::{Resource, ResourceID, RuntimeError};
use core::cmp::Ordering;
use core::fmt::Write;
use core::task::{Context, Poll};
use embedded_hal::digital::v2;
use nom_uri::{ToUri, Uri};

pub type ExtiEvent = dev::ExtiEvent;
pub type Channel = dev::Channel;
pub type Port = dev::Port;
pub type GpioEvent = dev::GpioEvent;

pub trait Device<InputError, OutputError> {
    fn init() -> Self;
    fn get_resource(uri: &str) -> Result<ResourceID, RuntimeError>;
    fn run(&mut self) -> !;
    fn get_sys(&self) -> &[&'static ()];
    fn get_input_pins(&self) -> &[&'static dyn v2::InputPin<Error = InputError>];
    fn get_output_pins(&self) -> &[&'static dyn v2::OutputPin<Error = OutputError>];
    fn get_pwm(&self) -> &[&'static ()];
    fn get_channels(&self) -> &[&'static ()];
    fn get_serials(&self) -> &[&'static ()];
    fn get_timers(&self) -> &[&'static ()];
}

/// Pin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pin {
    channel: Channel,
    port: Port,
}
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum TriggerEdge {
    Rising,
    Falling,
    All,
}

/// Should return the start of the heap allocation
/// In stm32f1 it startts at the data segment .uninit after .bss
#[inline]
pub fn heap_bottom() -> usize {
    dev::heap_bottom()
}

/// Wait for the next event
/// For ARM-Cortex-M3 cores this is the wfe (wait for event) instruction
#[inline]
pub fn sleep() {
    dev::sleep()
}

impl Pin {
    #[inline]
    pub fn new(channel: Channel, port: Port) -> Self {
        Self { channel, port }
    }
    #[inline]
    pub fn channel(&self) -> Channel {
        self.channel
    }
    #[inline]
    pub fn port(&self) -> Port {
        self.port
    }
}

pub struct Heap {
    size: usize,
}
pub struct SysClock {
    clock: usize,
}
//TODO: add a name as alias in uri
pub struct InputPin<HalPin: 'static> {
    id: Pin,
    resource: HalPin,
}
pub struct OutputPin<HalPin: 'static> {
    id: Pin,
    resource: HalPin,
}
pub struct PWMPin<HalPWMPin: 'static> {
    id: Pin,
    resource: HalPWMPin,
}

impl Resource for Heap {
    fn poll_read(
        &mut self,
        _context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        let parsed = self.size.to_string();
        let parsed = parsed.as_bytes();
        if buf.len() < parsed.len() {
            Poll::Ready(Err(io::Error::InvalidInput))
        } else {
            for i in 0..parsed.len() {
                buf[i] = parsed[i]
            }
            Poll::Ready(Ok(parsed.len()))
        }
    }
    fn poll_write(&mut self, _cx: &mut Context, _buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_flush(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_close(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(&mut self, _cx: &mut Context, _pos: SeekFrom) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
}
impl ToUri for Heap {
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(buffer, "Sys:heap").unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
    }
}
impl Heap {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl Resource for SysClock {
    fn poll_read(
        &mut self,
        _context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        let parsed = self.clock.to_string();
        let parsed = parsed.as_bytes();
        if buf.len() < parsed.len() {
            Poll::Ready(Err(io::Error::InvalidInput))
        } else {
            for i in 0..parsed.len() {
                buf[i] = parsed[i]
            }
            Poll::Ready(Ok(parsed.len()))
        }
    }
    fn poll_write(&mut self, _cx: &mut Context, _buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_flush(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_close(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(&mut self, _cx: &mut Context, _pos: SeekFrom) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
}
impl ToUri for SysClock {
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(buffer, "Sys:clock").unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
    }
}
impl SysClock {
    pub fn new(clock_in_hertz: usize) -> Self {
        Self {
            clock: clock_in_hertz,
        }
    }
}

impl<HalPin, Error> Resource for InputPin<HalPin>
where
    HalPin: v2::InputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.resource.is_high() {
            Ok(res) => {
                let res = res.to_string();
                let res = res.as_bytes();
                if buf.len() < res.len() {
                    return Poll::Ready(Err(io::Error::InvalidInput));
                }
                for i in 0..res.len() {
                    buf[i] = res[i]
                }
                Poll::Ready(Ok(res.len()))
            }
            Err(e) => {
                log::error!("{}", e);
                Poll::Ready(Err(io::Error::Other))
            }
        }
    }
    fn poll_write(&mut self, _cx: &mut Context, _buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_flush(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_close(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(&mut self, _cx: &mut Context, _pos: SeekFrom) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
}
impl<HalPin, Error> ToUri for InputPin<HalPin>
where
    HalPin: v2::InputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(
            buffer,
            "digital:gpio/p{}{}",
            self.id.channel(),
            self.id.port()
        )
        .unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
    }
}
impl<HalPin, Error> InputPin<HalPin>
where
    HalPin: v2::InputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin, hal_pin: HalPin) -> Self {
        InputPin {
            id: pin,
            resource: hal_pin,
        }
    }
}

impl<HalPin, Error> Resource for OutputPin<HalPin>
where
    HalPin: v2::OutputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_write(&mut self, _cx: &mut Context, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        for byte in buf {
            let res = match *byte != 0 {
                true => self.resource.set_high(),
                false => self.resource.set_low(),
            };
            if let Err(e) = &res {
                log::error!("{}", e);
                return Poll::Ready(Err(io::Error::Other));
            }
        }
        Poll::Ready(Ok(buf.len()))
    }
    fn poll_flush(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(&mut self, _cx: &mut Context, _pos: SeekFrom) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
}
impl<HalPin, Error> ToUri for OutputPin<HalPin>
where
    HalPin: v2::OutputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(
            buffer,
            "digital:gpio/p{}{}",
            self.id.channel(),
            self.id.port()
        )
        .unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
    }
}
impl<HalPin, Error> OutputPin<HalPin>
where
    HalPin: v2::OutputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin, hal_pin: HalPin) -> Self {
        OutputPin {
            id: pin,
            resource: hal_pin,
        }
    }
}

impl<HalPWMPin, Duty> Resource for PWMPin<HalPWMPin>
where
    HalPWMPin: embedded_hal::PwmPin<Duty = Duty>,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        unimplemented!()
    }
    fn poll_write(&mut self, _cx: &mut Context, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        unimplemented!()
    }
    fn poll_flush(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(&mut self, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(&mut self, _cx: &mut Context, _pos: SeekFrom) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
}
impl<HalPWMPin, Duty> ToUri for PWMPin<HalPWMPin>
where
    HalPWMPin: embedded_hal::PwmPin<Duty = Duty>,
{
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(
            buffer,
            "analog:pwm/p{}{}",
            self.id.channel(),
            self.id.port()
        )
        .unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
    }
}
impl<HalPWMPin, Duty> PWMPin<HalPWMPin>
where
    HalPWMPin: embedded_hal::PwmPin<Duty = Duty>,
{
    pub fn new(pin: Pin, hal_pin: HalPWMPin) -> Self {
        PWMPin {
            id: pin,
            resource: hal_pin,
        }
    }
}

impl Ord for Pin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.port
            .cmp(&other.port)
            .then(self.channel.cmp(&other.channel))
    }
}

impl PartialOrd for Pin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
