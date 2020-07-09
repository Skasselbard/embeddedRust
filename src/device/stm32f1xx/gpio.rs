use super::{Device, DeviceError, Pin};
use crate::events::{Event, Priority};
use crate::resources::{Resource, ResourceError};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use core::cmp::Ordering;
use core::convert::Infallible;
use embedded_hal::digital::v2;
use stm32f1xx_hal::device::interrupt;
use stm32f1xx_hal::gpio::{
    gpioa, gpiob, gpioc, gpiod, Alternate, Analog, ExtiPin, Floating, Input, OpenDrain, Output,
    PullDown, PullUp, PushPull, Pxx,
};

static mut GPIOS: Gpios = Gpios::new();
trait InputPin: v2::InputPin<Error = Infallible> {}
trait OutputPin: v2::StatefulOutputPin<Error = Infallible> + v2::OutputPin<Error = Infallible> {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ExtiEvent {
    Gpio(Pin),
    Pvd,
    RtcAlarm,
    UsbWakeup,
    EthernetWakeup,
}
#[derive(Eq, Clone, Copy)]
pub struct Gpio {
    id: Pin,
    direction: Direction,
    mode: PinMode,
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

pub struct Gpios {
    input: BTreeMap<Gpio, Box<dyn InputPin>>,
    output: BTreeMap<Gpio, Box<dyn OutputPin>>,
}

impl Gpios {
    const fn new() -> Self {
        Gpios {
            input: BTreeMap::new(),
            output: BTreeMap::new(),
        }
    }
}

macro_rules! check_interrupt {
    ($pinty:ty, $pinid:expr) => {
        // We can just reinterpret a null-tuple because the underlying
        // interrupt registers are determined by type.
        // No actual data is involved
        let mut pin = unsafe { core::mem::transmute::<(), $pinty>(()) };
        if pin.check_interrupt() {
            crate::events::push(
                Event::ExternalInterrupt(ExtiEvent::Gpio($pinid)),
                Priority::Critical,
            )
            .expect("filled event queue");
            pin.clear_interrupt_pending_bit();
        }
    };
}

/// Interrupt for GPIO Pin A0, B0 and C0
#[interrupt]
fn EXTI0() {
    check_interrupt!(gpioa::PA0<Input<Floating>>, Pin::PA0);
    check_interrupt!(gpiob::PB0<Input<Floating>>, Pin::PB0);
    check_interrupt!(gpioc::PC0<Input<Floating>>, Pin::PC0);
}

impl Resource for Gpio {
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
        match self.direction {
            Direction::Input => {
                if let Some(input_pin) = unsafe { GPIOS.input.get(self) } {
                    buf[0] = input_pin.is_high().unwrap().into(); // infallible
                } else {
                    return ResourceError::NotFound.into();
                }
            }
            Direction::Output => {
                if let Some(output_pin) = unsafe { GPIOS.output.get(self) } {
                    buf[0] = output_pin.is_set_high().unwrap().into(); // infallible
                } else {
                    return ResourceError::NotFound.into();
                }
            }
            Direction::Alternate => unimplemented!(),
        }
        Ok(1)
    }
    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
        match self.direction {
            Direction::Output => {
                if let Some(output_pin) = unsafe { GPIOS.output.get_mut(self) } {
                    for byte in buf {
                        match *byte != 0 {
                            true => output_pin.set_high().unwrap(), // infallible
                            false => output_pin.set_low().unwrap(), // infallible
                        }
                    }
                    Ok(buf.len())
                } else {
                    ResourceError::NotFound.into()
                }
            }
            Direction::Input => ResourceError::NonWritingResource.into(),
            Direction::Alternate => unimplemented!(),
        }
    }
    fn seek(&mut self, _: usize) -> nb::Result<(), ResourceError> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), ResourceError> {
        Ok(())
    }
}

#[macro_export]
macro_rules! init_gpio_generic {
    ($channel:ident, $pin:ident, $control_reg:ident, $direction:ident, $mode:ident) => {
        (
            crate::device::stm32f1xx::Pin::$pin,
            unsafe { core::mem::transmute::<(), stm32f1xx_hal::gpio::$channel::$control_reg>(()) },
            unsafe {
                core::mem::transmute::<(), stm32f1xx_hal::gpio::$channel::$pin<Input<$mode>>>(())
            },
        )
    };
}
#[macro_export]
macro_rules! init_gpio {
    (pa0, input, floating) => {
        init_gpio_generic!(gpioa, PA0, CRL, Input, Floating)
    };
}

#[macro_export]
macro_rules! build_gpio {
    ($device: ident, pa0, input, floating) => {{
            let (id, mut control_reg, pin) = init_gpio!(pa0, input, floating);
        if $device.is_used_pin(id) {
            panic!("multiple gpio creation")
        };
        let _ = pin.into_floating_input(&mut control_reg);
        let gpio = Gpio {
            id,
            mode: crate::device::stm32f1xx::PinMode::Floating,
            direction: crate::device::stm32f1xx::Direction::Input,
        };
        $device.reserve_pin(id);
        gpio
    }};
}
impl Gpio {
    fn new(pin: Pin, direction: Direction, mode: PinMode) -> Self {
        let gpio = Gpio {
            id: pin,
            direction,
            mode,
        };
        match pin{
            (Pin::PA0, Direction::Input, PinMode::Floating) =>
        }
        gpio
    }
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
