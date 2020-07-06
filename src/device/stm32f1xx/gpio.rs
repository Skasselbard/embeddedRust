use super::Pin;
use crate::events::{Event, Priority};
use crate::resources::{Resource, ResourceError};
use alloc::collections::{BTreeMap, BTreeSet};
use core::cmp::Ordering;
use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32f1xx_hal::device::interrupt;
use stm32f1xx_hal::gpio::{
    Alternate, Analog, ExtiPin, Floating, Input, OpenDrain, Output, PullDown, PullUp, PushPull, Pxx,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ExtiEvent {
    Gpio(Gpio),
    Pvd,
    RtcAlarm,
    UsbWakeup,
    EthernetWakeup,
}
#[derive(Eq, Clone, Copy)]
pub struct Gpio {
    pub id: Pin,
    pub direction: Direction,
    pub mode: PinMode,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Direction {
    Alternate,
    Input,
    Output,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum PinMode {
    Analog,
    Floating,
    OpenDrain,
    PullDown,
    PullUp,
    PushPull,
}

pub static mut GPIOS: Gpios = Gpios::new();

/// Intended to be a singleton
pub struct Gpios {
    pub analog: BTreeMap<Pin, Pxx<Analog>>,
    pub alternat_push_pull: BTreeMap<Pin, Pxx<Alternate<PushPull>>>,
    pub alternat_open_drain: BTreeMap<Pin, Pxx<Alternate<OpenDrain>>>,
    pub input_floating: BTreeMap<Pin, Pxx<Input<Floating>>>,
    pub input_pull_up: BTreeMap<Pin, Pxx<Input<PullUp>>>,
    pub input_pull_down: BTreeMap<Pin, Pxx<Input<PullDown>>>,
    pub output_open_drain: BTreeMap<Pin, Pxx<Output<OpenDrain>>>,
    pub output_push_pull: BTreeMap<Pin, Pxx<Output<PushPull>>>,
}

impl Gpios {
    pub const fn new() -> Self {
        Self {
            analog: BTreeMap::new(),
            alternat_push_pull: BTreeMap::new(),
            alternat_open_drain: BTreeMap::new(),
            input_floating: BTreeMap::new(),
            input_pull_up: BTreeMap::new(),
            input_pull_down: BTreeMap::new(),
            output_open_drain: BTreeMap::new(),
            output_push_pull: BTreeMap::new(),
        }
    }
    fn get_exti_pin(&mut self, gpio: &Gpio) -> Result<&mut dyn ExtiPin, ResourceError> {
        match &gpio.direction {
            Direction::Input => {
                let pin: &mut dyn ExtiPin = match &gpio.mode {
                    PinMode::PullUp => unsafe {
                        GPIOS
                            .input_pull_up
                            .get_mut(&gpio.id)
                            .ok_or(ResourceError::NotFound)?
                    },
                    PinMode::PullDown => unsafe {
                        GPIOS
                            .input_pull_down
                            .get_mut(&gpio.id)
                            .ok_or(ResourceError::NotFound)?
                    },
                    PinMode::Floating => unsafe {
                        GPIOS
                            .input_floating
                            .get_mut(&gpio.id)
                            .ok_or(ResourceError::NotFound)?
                    },
                    _ => unreachable!(),
                };
                Ok(pin)
            }
            _ => Err(ResourceError::ConfigurationError),
        }
    }
}
impl Gpio {
    pub fn enable_interrupt(
        &mut self,
        afio: &mut stm32f1xx_hal::afio::Parts,
    ) -> Result<(), ResourceError> {
        let pxx = unsafe { GPIOS.get_exti_pin(self)? };
        pxx.make_interrupt_source(afio);
        Ok(())
    }
    pub fn disable_interrupt(
        &mut self,
        exti: &stm32f1xx_hal::device::EXTI,
    ) -> Result<(), ResourceError> {
        let pxx = unsafe { GPIOS.get_exti_pin(self)? };
        pxx.disable_interrupt(exti);
        Ok(())
    }
}
static mut EXTIPINS: BTreeSet<Gpio> = BTreeSet::new();

/// Interrupt for GPIO Pin A0, B0 and C0
#[interrupt]
fn EXTI0() {
    // Consider all possible pins for this EXTI source
    for pin in unsafe { &mut EXTIPINS.iter() } {
        // retrieve current bit from ``GPIOS``
        let pxx = unsafe { GPIOS.get_exti_pin(pin).expect("Exti pin lookup error") };
        // check pin for interrupt source
        if pxx.check_interrupt() {
            crate::events::push(
                Event::ExternalInterrupt(ExtiEvent::Gpio(*pin)),
                Priority::Critical,
            )
            .expect("filled event queue");
            pxx.clear_interrupt_pending_bit();
        }
    }
}

impl Resource for Gpio {
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
        // FIXME: probably not thread safe
        let gpio: &dyn InputPin<Error = Infallible> = match self.direction {
            Direction::Input => match self.mode {
                PinMode::PullUp => unsafe {
                    GPIOS
                        .input_pull_up
                        .get(&self.id)
                        .ok_or(ResourceError::NotFound)?
                },
                PinMode::PullDown => unsafe {
                    GPIOS
                        .input_pull_down
                        .get(&self.id)
                        .ok_or(ResourceError::NotFound)?
                },
                PinMode::Floating => unsafe {
                    GPIOS
                        .input_floating
                        .get(&self.id)
                        .ok_or(ResourceError::NotFound)?
                },
                _ => unreachable!(),
            },
            _ => return ResourceError::NonReadingResource.into(),
        };
        buf[0] = gpio.is_high().unwrap().into(); // infallible
        Ok(1)
    }
    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
        let gpio: &mut dyn OutputPin<Error = Infallible> = match self.direction {
            Direction::Output => match self.mode {
                PinMode::OpenDrain => unsafe {
                    GPIOS
                        .output_open_drain
                        .get_mut(&self.id)
                        .ok_or(ResourceError::NotFound)?
                },
                PinMode::PushPull => unsafe {
                    GPIOS
                        .output_push_pull
                        .get_mut(&self.id)
                        .ok_or(ResourceError::NotFound)?
                },
                _ => unreachable!(),
            },
            _ => return ResourceError::NonReadingResource.into(),
        };
        for byte in buf {
            match *byte != 0 {
                true => gpio.set_high().unwrap(), // infallible
                false => gpio.set_low().unwrap(), // infallible
            }
        }
        Ok(buf.len())
    }
    fn seek(&mut self, _: usize) -> nb::Result<(), ResourceError> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), ResourceError> {
        Ok(())
    }
}

#[macro_export]
macro_rules! build_gpio {
    ($channel:ident, pa0, input, floating) => {{
        use embedded_rust::device::stm32f1xx::*;
        let pin = $channel.pa0.into_floating_input(&mut $channel.crl);
        let id = Pin::PA0;
        unsafe { GPIOS.input_floating.insert(id, pin.downgrade()) };
        Gpio {
            id,
            mode: PinMode::Floating,
            direction: Direction::Input,
        }
    }};
    ($channel:ident, pa6, input, floating) => {{
        use embedded_rust::device::stm32f1xx::*;
        let pin = $channel.pa6.into_floating_input(&mut $channel.crl);
        let id = Pin::PA6;
        unsafe { GPIOS.input_floating.insert(id, pin.downgrade()) };
        Gpio {
            id,
            mode: PinMode::Floating,
            direction: Direction::Input,
        }
    }};
    ($channel:ident, pa7, input, floating) => {{
        use embedded_rust::device::stm32f1xx::*;
        let pin = $channel.pa7.into_floating_input(&mut $channel.crl);
        let id = Pin::PA7;
        unsafe { GPIOS.input_floating.insert(id, pin.downgrade()) };
        Gpio {
            id,
            mode: PinMode::Floating,
            direction: Direction::Input,
        }
    }};
}

impl Ord for Gpio {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Gpio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Gpio {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
