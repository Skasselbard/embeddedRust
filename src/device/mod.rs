mod cortex_m;
pub mod pwm;
pub mod usart;

#[cfg(feature = "stm32f1xx")]
mod stm32f1xx;
#[cfg(feature = "stm32f1xx")]
pub(crate) use self::cortex_m::init_heap;
#[cfg(feature = "stm32f1xx")]
use stm32f1xx as dev;

use crate::resources::ResourceError;
use crate::{Resource, ResourceID, RuntimeError, Task};
use core::cmp::Ordering;
use core::task::{Context, Poll};
use embedded_hal::digital::v2;
use nom_uri::ToUri;

pub type ComponentConfiguration = dev::ComponentConfiguration;
pub type ExtiEvent = dev::ExtiEvent;
pub type PinMode = dev::PinMode;
pub type Direction = dev::Direction;
pub type Gpio = dev::Gpio;
pub type Channel = dev::Channel;
pub type Port = dev::Port;

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
    index: u8,
}
pub struct SysClock {
    clock: usize,
    index: u8,
}
pub struct InputPin<Pin: 'static>(Pin);
pub struct OutputPin<Pin: 'static>(Pin);

impl Resource for Heap {
    fn read_next(&mut self, _: &mut Context) -> Poll<Option<u8>> {
        let byte = self.size.to_be_bytes()[self.index as usize];
        self.index = (self.index + 1) % core::mem::size_of::<usize>() as u8;
        Poll::Ready(Some(byte))
    }
    fn seek(
        &mut self,
        _: &mut core::task::Context<'_>,
        _: usize,
    ) -> core::task::Poll<core::result::Result<(), ResourceError>> {
        Poll::Ready(Ok(()))
    }
}
impl Heap {
    pub fn new(size: usize) -> Self {
        Self { size, index: 0 }
    }
}
impl Resource for SysClock {
    fn read_next(&mut self, _: &mut Context) -> Poll<Option<u8>> {
        let byte = self.clock.to_be_bytes()[self.index as usize];
        self.index = (self.index + 1) % core::mem::size_of::<usize>() as u8;
        Poll::Ready(Some(byte))
    }
    fn seek(
        &mut self,
        _: &mut core::task::Context<'_>,
        _: usize,
    ) -> core::task::Poll<core::result::Result<(), ResourceError>> {
        Poll::Ready(Ok(()))
    }
}
impl SysClock {
    pub fn new(clock_in_hertz: usize) -> Self {
        Self {
            clock: clock_in_hertz,
            index: 0,
        }
    }
}

// Dummy Resource. Can be used as a placeholder.
impl Resource for () {
    fn seek(
        &mut self,
        _: &mut core::task::Context<'_>,
        _: usize,
    ) -> core::task::Poll<core::result::Result<(), ResourceError>> {
        Poll::Ready(Ok(()))
    }
}

impl<Pin, Error> Resource for InputPin<Pin>
where
    Pin: v2::InputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn read_next(&mut self, _: &mut Context) -> Poll<Option<u8>> {
        match self.0.is_high() {
            Ok(res) => Poll::Ready(Some(res as u8)),
            Err(e) => {
                log::error!("{}", e);
                Poll::Ready(None)
            }
        }
    }
    fn seek(
        &mut self,
        _: &mut core::task::Context<'_>,
        _: usize,
    ) -> core::task::Poll<core::result::Result<(), ResourceError>> {
        Poll::Ready(Ok(()))
    }
}
impl<Pin, Error> InputPin<Pin>
where
    Pin: v2::InputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin) -> Self {
        InputPin(pin)
    }
}

impl<Pin, Error> Resource for OutputPin<Pin>
where
    Pin: v2::OutputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn write_next(&mut self, context: &mut Context, byte: u8) -> Poll<Result<(), ResourceError>> {
        let res = match byte != 0 {
            true => self.0.set_high(),
            false => self.0.set_low(),
        };
        if let Err(e) = &res {
            log::error!("{}", e);
        }
        Poll::Ready(res.map_err(|_| ResourceError::WriteError))
    }
    fn seek(
        &mut self,
        _: &mut core::task::Context<'_>,
        _: usize,
    ) -> core::task::Poll<core::result::Result<(), ResourceError>> {
        Poll::Ready(Ok(()))
    }
}
impl<Pin, Error> OutputPin<Pin>
where
    Pin: v2::OutputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin) -> Self {
        OutputPin(pin)
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
