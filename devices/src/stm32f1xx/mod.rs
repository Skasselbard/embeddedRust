
#[macro_use]
mod usart;
mod gpio;
mod pwm;
pub use gpio::*;
pub use pwm::*;
use stm32f1xx_hal::device::USART1;
use stm32f1xx_hal::serial::{Event, Serial};
pub use usart::*;

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use core::cmp::Ordering;
use stm32f1xx_hal::{
    adc,
    device::interrupt,
    pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

pub enum ComponentConfiguration {
    Clock,
    Gpio(Gpio),
    Usart,
    Pwm,
}

/// Pin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pin {
    channel: Channel,
    port: Port,
}

impl Pin {
    pub fn new(channel: Channel, port: Port) -> Self {
        Self { channel, port }
    }
    pub fn channel(&self) -> Channel {
        self.channel
    }
    pub fn port(&self) -> Port {
        self.port
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
/// consider configuring clocks before adc construction:
/// ```
/// let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);
/// ```
/// ## return
/// (adc, (channel1, channel2, .. ))
#[macro_export]
macro_rules! adc1 {
    ($gpioa:expr, $peripherals:expr, $rcc:expr, $clocks:expr, $($pina:expr),+) => {{
        let adc = adc::Adc::adc1($peripherals.ADC1, &mut $rcc.apb2, $clocks);
        (adc, ($($pina.into_analog(&mut $gpioa.crl)),+))
    }};
}
