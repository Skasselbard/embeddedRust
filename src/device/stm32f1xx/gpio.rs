use crate::device::{ExtiEvent, Pin};
use crate::events::{self, Event};
use crate::io::{self, SeekFrom};
use crate::resources::Resource;
use crate::resources::StrWriter;
use crate::schemes::{Scheme, Virtual};
use crate::{Runtime, RuntimeError};
use core::fmt::Write;
use core::str::FromStr;
use core::task::{Context, Poll};
use nom_uri::{ToUri, Uri};
use stm32f1xx_hal::device::interrupt;
use stm32f1xx_hal::gpio::{gpioa, gpiob, gpioc, ExtiPin, Floating, Input};

macro_rules! check_interrupt {
    ($pinty:ty, $channel:expr, $port:expr) => {
        // We can just reinterpret a null-tuple because the underlying
        // interrupt registers are determined by type.
        // No actual data is involved
        let mut pin = unsafe { core::mem::transmute::<(), $pinty>(()) };
        if pin.check_interrupt() {
            let e = Event::ExternalInterrupt(ExtiEvent::Gpio(Pin {
                channel: $channel,
                port: $port,
            }));
            cortex_m::interrupt::free(|cs| {
                events::push(e, cs);
                pin.clear_interrupt_pending_bit();
            });
        }
    };
}

// TODO: other EXTI lines
// Interrupt for GPIO Pin A0, B0 and C0
#[interrupt]
fn EXTI0() {
    //TODO: Channel D and E
    check_interrupt!(gpioa::PA0<Input<Floating>>, Channel::A, Port::P00);
    check_interrupt!(gpiob::PB0<Input<Floating>>, Channel::B, Port::P00);
    check_interrupt!(gpioc::PC0<Input<Floating>>, Channel::C, Port::P00);
}
/// Resource that acts as a sink for GPIO events
pub struct GpioEvent {
    id: Pin,
    event: bool,
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

impl GpioEvent {
    pub fn from_uri(uri: &str) -> Result<Self, RuntimeError> {
        let mut parsed_uri = Uri::parse(uri).unwrap();
        if let Ok(Scheme::V(Virtual::Event)) = Scheme::from_str(parsed_uri.scheme()) {
            parsed_uri.set_scheme("digital").unwrap();
            let mut buf_array = [0u8; 255];
            if let Ok(_) = Runtime::get().get_resource(parsed_uri.as_str(&mut buf_array).unwrap()) {
                return Ok(Self {
                    event: false,
                    id: pin_from_uri(uri)?,
                });
            }
        }
        Err(RuntimeError::ResourceNotFound)
    }
}

fn pin_from_uri(uri: &str) -> Result<Pin, RuntimeError> {
    // scheme:gpio/pb13 -> pb13
    let pin_str = uri.rsplit("/").next().unwrap();
    // pb13 -> b
    let channel = match pin_str.chars().nth(1).unwrap() {
        'a' | 'A' => Channel::A,
        'b' | 'B' => Channel::B,
        'c' | 'C' => Channel::C,
        'd' | 'D' => Channel::D,
        'e' | 'E' => Channel::E,
        _ => return Err(RuntimeError::UriParseError),
    };
    // pb13 -> 13
    let port = match pin_str {
        s if s.ends_with("0") || s.ends_with("00") => Port::P00,
        s if s.ends_with("1") || s.ends_with("01") => Port::P01,
        s if s.ends_with("2") || s.ends_with("02") => Port::P02,
        s if s.ends_with("3") || s.ends_with("03") => Port::P03,
        s if s.ends_with("4") || s.ends_with("04") => Port::P04,
        s if s.ends_with("5") || s.ends_with("05") => Port::P05,
        s if s.ends_with("6") || s.ends_with("06") => Port::P06,
        s if s.ends_with("7") || s.ends_with("07") => Port::P07,
        s if s.ends_with("8") || s.ends_with("08") => Port::P08,
        s if s.ends_with("9") || s.ends_with("09") => Port::P09,
        s if s.ends_with("10") => Port::P10,
        s if s.ends_with("11") => Port::P11,
        s if s.ends_with("12") => Port::P12,
        s if s.ends_with("13") => Port::P13,
        s if s.ends_with("14") => Port::P14,
        s if s.ends_with("15") => Port::P15,
        _ => return Err(RuntimeError::UriParseError),
    };
    Ok(Pin::new(channel, port))
}

impl Resource for GpioEvent {
    fn poll_read(
        &mut self,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        if self.event {
            self.event = false;
            buf[0] = 0 as u8;
            Poll::Ready(Ok(1))
        } else {
            Runtime::get().register_waker(
                &Event::ExternalInterrupt(ExtiEvent::Gpio(self.id)),
                context.waker(),
            );
            self.event = true; //TODO: reset im waker?
            Poll::Pending
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
impl ToUri for GpioEvent {
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        let mut buffer = StrWriter::from(buffer);
        write!(
            buffer,
            "event:gpio/p{}{}",
            self.id.channel(),
            self.id.port()
        )
        .unwrap();
        Uri::parse(buffer.buffer().unwrap()).unwrap()
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
