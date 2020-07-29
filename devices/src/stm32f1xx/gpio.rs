use super::{ComponentConfiguration, Pin};
use crate::events::Event;
use crate::resources::{RegisterComponent, Resource, ResourceError};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use core::cmp::Ordering;
use core::convert::Infallible;
use embedded_hal::digital::v2;
use nom_uri::{ToUri, Uri};
use stm32f1xx_hal::device::interrupt;
use stm32f1xx_hal::gpio::{
    gpioa, gpiob, gpioc, gpiod, Alternate, Analog, ExtiPin, Floating, Input, OpenDrain, Output,
    PullDown, PullUp, PushPull, Pxx,
};

static mut GPIOS: Gpios = Gpios::new();

pub trait InputPin: v2::InputPin<Error = Infallible> {}
pub trait OutputPin:
    v2::StatefulOutputPin<Error = Infallible> + v2::OutputPin<Error = Infallible>
{
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ExtiEvent {
    Gpio(Pin, bool),
    Pvd,
    RtcAlarm,
    UsbWakeup,
    EthernetWakeup,
}
#[derive(Eq, Clone, Copy, Debug, Hash)]
pub struct Gpio {
    id: Pin,
    direction: Direction,
    mode: PinMode,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord, Hash)]
pub enum Channel {
    A,
    B,
    C,
    D,
    E,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord, Hash)]
pub enum Port {
    P00,
    P01,
    P02,
    P03,
    P04,
    P05,
    P06,
    P07,
    P08,
    P09,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Direction {
    Alternate,
    Input(Option<TriggerEdge>),
    Output,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum TriggerEdge {
    Rising,
    Falling,
    All,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
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

impl Gpio {
    pub fn new(pin: Pin, direction: Direction, mode: PinMode) -> Self {
        Gpio {
            id: pin,
            direction,
            mode,
        }
    }
    pub fn id(&self) -> Pin {
        self.id
    }
    pub fn channel(&self) -> Channel {
        self.id.channel()
    }
    pub fn port(&self) -> Port {
        self.id.port()
    }
    pub fn direction(&self) -> Direction {
        self.direction
    }
    pub fn mode(&self) -> PinMode {
        self.mode
    }
}
impl Gpios {
    const fn new() -> Self {
        Gpios {
            input: BTreeMap::new(),
            output: BTreeMap::new(),
        }
    }
}

//FIXME: handle interrrupts
macro_rules! check_interrupt {
    ($pinty:ty, $pinid:expr) => {
        // We can just reinterpret a null-tuple because the underlying
        // interrupt registers are determined by type.
        // No actual data is involved
        let mut pin = unsafe { core::mem::transmute::<(), $pinty>(()) };
        if pin.check_interrupt() {
            // crate::events::push(
            //     Event::ExternalInterrupt(ExtiEvent::Gpio($pinid)),
            //     Priority::Critical,
            // )
            // .expect("filled event queue");
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

//////////////////////////////////////////
// impls for component registration
impl<Mode> InputPin for Pxx<Input<Mode>> {}
impl<Mode> OutputPin for Pxx<Output<Mode>> {}
impl<Mode> RegisterComponent for Pxx<Input<Mode>>
where
    Pxx<Input<Mode>>: Sized + InputPin + 'static,
{
    fn register_component(self, configuration: ComponentConfiguration) {
        let key = match configuration {
            ComponentConfiguration::Gpio(gpio) => gpio,
            _ => panic!("gpio has non gpio configuration"),
        };
        unsafe { GPIOS.input.insert(key, Box::new(self)) };
    }
}
impl<Mode> RegisterComponent for Pxx<Output<Mode>>
where
    Pxx<Output<Mode>>: Sized + OutputPin + 'static,
{
    fn register_component(self, configuration: ComponentConfiguration) {
        let key = match configuration {
            ComponentConfiguration::Gpio(gpio) => gpio,
            _ => panic!("gpio has non gpio configuration"),
        };
        unsafe { GPIOS.output.insert(key, Box::new(self)) };
    }
}
impl RegisterComponent for Pxx<Analog>
where
    Pxx<Analog>: Sized + 'static,
{
    fn register_component(self, _configuration: ComponentConfiguration) {
        unimplemented!()
    }
}
/////////////////////////////////////////////

impl Resource for Gpio {
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
        match self.direction {
            Direction::Input(_) => {
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
            Direction::Input(_) => ResourceError::NonWritingResource.into(),
            Direction::Alternate => unimplemented!(),
        }
    }
    fn seek(&mut self, _: usize) -> nb::Result<(), ResourceError> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), ResourceError> {
        Ok(())
    }
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        (self as &dyn ToUri).to_uri(buffer)
    }
}

impl<'uri> ToUri<'uri> for Gpio {
    fn to_uri(&self, buffer: &'uri mut str) -> Uri<'uri> {
        use crate::resources::StrWriter;
        use core::fmt::Write;
        let scheme = match self.mode() {
            PinMode::Analog => "analog",
            _ => "digital",
        };
        let mut buffer = StrWriter::from(buffer);
        write!(buffer, "{}:gpio/p{}{}", scheme, self.channel(), self.port())
            .expect("format error for gpio uri");
        Uri::parse(buffer.buffer().expect("format error for gpio uri")).unwrap()
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

impl core::fmt::Display for Channel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Channel::A => write!(f, "a"),
            Channel::B => write!(f, "b"),
            Channel::C => write!(f, "c"),
            Channel::D => write!(f, "d"),
            Channel::E => write!(f, "e"),
        }
    }
}
impl core::fmt::Display for Port {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Port::P00 => write!(f, "0"),
            Port::P01 => write!(f, "1"),
            Port::P02 => write!(f, "2"),
            Port::P03 => write!(f, "3"),
            Port::P04 => write!(f, "4"),
            Port::P05 => write!(f, "5"),
            Port::P06 => write!(f, "6"),
            Port::P07 => write!(f, "7"),
            Port::P08 => write!(f, "8"),
            Port::P09 => write!(f, "9"),
            Port::P10 => write!(f, "10"),
            Port::P11 => write!(f, "11"),
            Port::P12 => write!(f, "12"),
            Port::P13 => write!(f, "13"),
            Port::P14 => write!(f, "14"),
            Port::P15 => write!(f, "15"),
        }
    }
}
