use crate::events::{self, Event};
use crate::{device::ExtiEvent, resources::gpio::Pin};
use stm32f1xx_hal::device::interrupt;
use stm32f1xx_hal::gpio::{gpioa, gpiob, gpioc, gpiod, gpioe, ExtiPin, Floating, Input};

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

// fn pin_from_uri(uri: &str) -> Result<Pin, RuntimeError> {
//     // scheme:gpio/pb13 -> pb13
//     let pin_str = uri.rsplit("/").next().unwrap();
//     // pb13 -> b
//     let channel = match pin_str.chars().nth(1).unwrap() {
//         'a' | 'A' => Channel::A,
//         'b' | 'B' => Channel::B,
//         'c' | 'C' => Channel::C,
//         'd' | 'D' => Channel::D,
//         'e' | 'E' => Channel::E,
//         _ => return Err(RuntimeError::UriParseError),
//     };
//     // pb13 -> 13
//     let port = match pin_str {
//         s if s.ends_with("0") || s.ends_with("00") => Port::P00,
//         s if s.ends_with("1") || s.ends_with("01") => Port::P01,
//         s if s.ends_with("2") || s.ends_with("02") => Port::P02,
//         s if s.ends_with("3") || s.ends_with("03") => Port::P03,
//         s if s.ends_with("4") || s.ends_with("04") => Port::P04,
//         s if s.ends_with("5") || s.ends_with("05") => Port::P05,
//         s if s.ends_with("6") || s.ends_with("06") => Port::P06,
//         s if s.ends_with("7") || s.ends_with("07") => Port::P07,
//         s if s.ends_with("8") || s.ends_with("08") => Port::P08,
//         s if s.ends_with("9") || s.ends_with("09") => Port::P09,
//         s if s.ends_with("10") => Port::P10,
//         s if s.ends_with("11") => Port::P11,
//         s if s.ends_with("12") => Port::P12,
//         s if s.ends_with("13") => Port::P13,
//         s if s.ends_with("14") => Port::P14,
//         s if s.ends_with("15") => Port::P15,
//         _ => return Err(RuntimeError::UriParseError),
//     };
//     Ok(Pin::new(channel, port))
// }

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

macro_rules! check_interrupt {
    ($pinty:ty, $channel:expr, $port:expr) => {
        // We can just reinterpret a null-tuple because the underlying
        // interrupt registers are determined by type.
        // No actual data is involved
        let mut pin = unsafe { core::mem::transmute::<(), $pinty>(()) };
        if pin.check_interrupt() {
            let e = Event::ExternalInterrupt(ExtiEvent::Gpio(Pin::new($channel, $port)));
            cortex_m::interrupt::free(|cs| {
                events::push(e, cs);
                pin.clear_interrupt_pending_bit();
            });
        }
    };
}

#[interrupt]
fn EXTI0() {
    check_interrupt!(gpioa::PA0<Input<Floating>>, Channel::A, Port::P00);
    check_interrupt!(gpiob::PB0<Input<Floating>>, Channel::B, Port::P00);
    check_interrupt!(gpioc::PC0<Input<Floating>>, Channel::C, Port::P00);
    check_interrupt!(gpiod::PD0<Input<Floating>>, Channel::D, Port::P00);
    check_interrupt!(gpioe::PE0<Input<Floating>>, Channel::E, Port::P00);
}
#[interrupt]
fn EXTI1() {
    check_interrupt!(gpioa::PA1<Input<Floating>>, Channel::A, Port::P01);
    check_interrupt!(gpiob::PB1<Input<Floating>>, Channel::B, Port::P01);
    check_interrupt!(gpioc::PC1<Input<Floating>>, Channel::C, Port::P01);
    check_interrupt!(gpiod::PD1<Input<Floating>>, Channel::D, Port::P01);
    check_interrupt!(gpioe::PE1<Input<Floating>>, Channel::E, Port::P01);
}
#[interrupt]
fn EXTI2() {
    check_interrupt!(gpioa::PA2<Input<Floating>>, Channel::A, Port::P02);
    check_interrupt!(gpiob::PB2<Input<Floating>>, Channel::B, Port::P02);
    check_interrupt!(gpioc::PC2<Input<Floating>>, Channel::C, Port::P02);
    check_interrupt!(gpiod::PD2<Input<Floating>>, Channel::D, Port::P02);
    check_interrupt!(gpioe::PE2<Input<Floating>>, Channel::E, Port::P02);
}
#[interrupt]
fn EXTI3() {
    check_interrupt!(gpioa::PA3<Input<Floating>>, Channel::A, Port::P03);
    check_interrupt!(gpiob::PB3<Input<Floating>>, Channel::B, Port::P03);
    check_interrupt!(gpioc::PC3<Input<Floating>>, Channel::C, Port::P03);
    check_interrupt!(gpiod::PD3<Input<Floating>>, Channel::D, Port::P03);
    check_interrupt!(gpioe::PE3<Input<Floating>>, Channel::E, Port::P03);
}
#[interrupt]
fn EXTI4() {
    check_interrupt!(gpioa::PA4<Input<Floating>>, Channel::A, Port::P04);
    check_interrupt!(gpiob::PB4<Input<Floating>>, Channel::B, Port::P04);
    check_interrupt!(gpioc::PC4<Input<Floating>>, Channel::C, Port::P04);
    check_interrupt!(gpiod::PD4<Input<Floating>>, Channel::D, Port::P04);
    check_interrupt!(gpioe::PE4<Input<Floating>>, Channel::E, Port::P04);
}
#[interrupt]
fn EXTI9_5() {
    check_interrupt!(gpioa::PA5<Input<Floating>>, Channel::A, Port::P05);
    check_interrupt!(gpiob::PB5<Input<Floating>>, Channel::B, Port::P05);
    check_interrupt!(gpioc::PC5<Input<Floating>>, Channel::C, Port::P05);
    check_interrupt!(gpiod::PD5<Input<Floating>>, Channel::D, Port::P05);
    check_interrupt!(gpioe::PE5<Input<Floating>>, Channel::E, Port::P05);
    check_interrupt!(gpioa::PA6<Input<Floating>>, Channel::A, Port::P06);
    check_interrupt!(gpiob::PB6<Input<Floating>>, Channel::B, Port::P06);
    check_interrupt!(gpioc::PC6<Input<Floating>>, Channel::C, Port::P06);
    check_interrupt!(gpiod::PD6<Input<Floating>>, Channel::D, Port::P06);
    check_interrupt!(gpioe::PE6<Input<Floating>>, Channel::E, Port::P06);
    check_interrupt!(gpioa::PA7<Input<Floating>>, Channel::A, Port::P07);
    check_interrupt!(gpiob::PB7<Input<Floating>>, Channel::B, Port::P07);
    check_interrupt!(gpioc::PC7<Input<Floating>>, Channel::C, Port::P07);
    check_interrupt!(gpiod::PD7<Input<Floating>>, Channel::D, Port::P07);
    check_interrupt!(gpioe::PE7<Input<Floating>>, Channel::E, Port::P07);
    check_interrupt!(gpioa::PA8<Input<Floating>>, Channel::A, Port::P08);
    check_interrupt!(gpiob::PB8<Input<Floating>>, Channel::B, Port::P08);
    check_interrupt!(gpioc::PC8<Input<Floating>>, Channel::C, Port::P08);
    check_interrupt!(gpiod::PD8<Input<Floating>>, Channel::D, Port::P08);
    check_interrupt!(gpioe::PE8<Input<Floating>>, Channel::E, Port::P08);
    check_interrupt!(gpioa::PA9<Input<Floating>>, Channel::A, Port::P09);
    check_interrupt!(gpiob::PB9<Input<Floating>>, Channel::B, Port::P09);
    check_interrupt!(gpioc::PC9<Input<Floating>>, Channel::C, Port::P09);
    check_interrupt!(gpiod::PD9<Input<Floating>>, Channel::D, Port::P09);
    check_interrupt!(gpioe::PE9<Input<Floating>>, Channel::E, Port::P09);
}
#[interrupt]
fn EXTI15_10() {
    check_interrupt!(gpioa::PA10<Input<Floating>>, Channel::A, Port::P10);
    check_interrupt!(gpiob::PB10<Input<Floating>>, Channel::B, Port::P10);
    check_interrupt!(gpioc::PC10<Input<Floating>>, Channel::C, Port::P10);
    check_interrupt!(gpiod::PD10<Input<Floating>>, Channel::D, Port::P10);
    check_interrupt!(gpioe::PE10<Input<Floating>>, Channel::E, Port::P10);
    check_interrupt!(gpioa::PA11<Input<Floating>>, Channel::A, Port::P11);
    check_interrupt!(gpiob::PB11<Input<Floating>>, Channel::B, Port::P11);
    check_interrupt!(gpioc::PC11<Input<Floating>>, Channel::C, Port::P11);
    check_interrupt!(gpiod::PD11<Input<Floating>>, Channel::D, Port::P11);
    check_interrupt!(gpioe::PE11<Input<Floating>>, Channel::E, Port::P11);
    check_interrupt!(gpioa::PA12<Input<Floating>>, Channel::A, Port::P12);
    check_interrupt!(gpiob::PB12<Input<Floating>>, Channel::B, Port::P12);
    check_interrupt!(gpioc::PC12<Input<Floating>>, Channel::C, Port::P12);
    check_interrupt!(gpiod::PD12<Input<Floating>>, Channel::D, Port::P12);
    check_interrupt!(gpioe::PE12<Input<Floating>>, Channel::E, Port::P12);
    check_interrupt!(gpioa::PA13<Input<Floating>>, Channel::A, Port::P13);
    check_interrupt!(gpiob::PB13<Input<Floating>>, Channel::B, Port::P13);
    check_interrupt!(gpioc::PC13<Input<Floating>>, Channel::C, Port::P13);
    check_interrupt!(gpiod::PD13<Input<Floating>>, Channel::D, Port::P13);
    check_interrupt!(gpioe::PE13<Input<Floating>>, Channel::E, Port::P13);
    check_interrupt!(gpioa::PA14<Input<Floating>>, Channel::A, Port::P14);
    check_interrupt!(gpiob::PB14<Input<Floating>>, Channel::B, Port::P14);
    check_interrupt!(gpioc::PC14<Input<Floating>>, Channel::C, Port::P14);
    check_interrupt!(gpiod::PD14<Input<Floating>>, Channel::D, Port::P14);
    check_interrupt!(gpioe::PE14<Input<Floating>>, Channel::E, Port::P14);
    check_interrupt!(gpioa::PA15<Input<Floating>>, Channel::A, Port::P15);
    check_interrupt!(gpiob::PB15<Input<Floating>>, Channel::B, Port::P15);
    check_interrupt!(gpioc::PC15<Input<Floating>>, Channel::C, Port::P15);
    check_interrupt!(gpiod::PD15<Input<Floating>>, Channel::D, Port::P15);
    check_interrupt!(gpioe::PE15<Input<Floating>>, Channel::E, Port::P15);
}
