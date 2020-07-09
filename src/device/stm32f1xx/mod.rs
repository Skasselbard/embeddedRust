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
use proc_macro::TokenStream;
use stm32f1xx_hal::{
    adc,
    device::interrupt,
    pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

#[derive(Debug, Copy, Clone)]
pub enum DeviceError {
    AlreadyInUse,
}

pub struct Device {
    flash: stm32f1xx_hal::flash::Parts,
    clocks: stm32f1xx_hal::rcc::Clocks,
    ahb: stm32f1xx_hal::rcc::AHB,
    apb1: stm32f1xx_hal::rcc::APB1,
    apb2: stm32f1xx_hal::rcc::APB2,
    bkp: stm32f1xx_hal::rcc::BKP,
    gpioa: stm32f1xx_hal::gpio::gpioa::Parts,
    gpiob: stm32f1xx_hal::gpio::gpiob::Parts,
    afio: stm32f1xx_hal::afio::Parts,
    usart1: stm32f1xx_hal::stm32::USART1,
    usart2: stm32f1xx_hal::stm32::USART2,
    usart3: stm32f1xx_hal::stm32::USART3,
    occupied_pins: BTreeSet<Pin>,
}

// TODO: handle GPIOs
// TODO: Probably can be handled dynamically in gpio lists
// TODO: maybe can be handled statically by casting from nothing (e.g. PA0 -> Gpio{id:Pin::PA0, ..})
// TODO: Pins are only marker structs => keeping track of already generated gpios should be sufficient
impl Device {
    pub fn new() -> Self {
        let peripherals = stm32f1xx_hal::pac::Peripherals::take().unwrap();
        let mut flash = peripherals.FLASH.constrain();
        let rcc = peripherals.RCC.constrain();
        let cfgr = rcc.cfgr;
        let clocks = cfgr.freeze(&mut flash.acr);
        let ahb = rcc.ahb;
        let apb1 = rcc.apb1;
        let mut apb2 = rcc.apb2;
        let bkp = rcc.bkp;

        let gpioa = peripherals.GPIOA.split(&mut apb2);
        let gpiob = peripherals.GPIOB.split(&mut apb2);
        let afio = peripherals.AFIO.constrain(&mut apb2);
        Self {
            // peripherals,
            flash,
            ahb,
            apb1,
            apb2,
            bkp,
            clocks,
            gpioa,
            gpiob,
            afio,

            usart1: peripherals.USART1,
            usart2: peripherals.USART2,
            usart3: peripherals.USART3,

            occupied_pins: BTreeSet::new(),
        }
    }

    pub fn set_up_gpio(&mut self, pin: Pin, direction: Direction, mode: PinMode) {
        unimplemented!()
    }

    pub fn is_used_pin(&self, pin: Pin) -> bool {
        self.occupied_pins.contains(&pin)
    }
    pub fn reserve_pin(&mut self, pin: Pin) {
        assert!(!self.is_used_pin(pin));
        self.occupied_pins.insert(pin);
    }
    // pub fn usart1(&mut self){ //-> crate::device::usart::Usart<USART1> {
    //     let tx = self.gpioa.pa9.into_alternate_push_pull(&mut self.gpioa.crh);
    //     let rx = self.gpioa.pa10;
    //     let mut serial = Serial::usart1(
    //         self.usart1,
    //         (tx, rx),
    //         &mut self.afio.mapr,
    //         stm32f1xx_hal::serial::Config::default(),
    //         self.clocks,
    //         &mut self.apb2,
    //     );
    //     serial.listen(Event::Rxne);
    //     let (tx, rx) = serial.split();
    //     let serial = Usart::new(tx, rx).unwrap();
    //     //crate::device::usart::Usart::new(serial)
    // }
}

/// Pin ID
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Pin {
    PA0,
    PA1,
    PA2,
    PA3,
    PA4,
    PA5,
    PA6,
    PA7,
    PA8,
    PA9,
    PA10,
    PA11,
    PA12,
    PA13,
    PA14,
    PA15,
    PB0,
    PB1,
    PB2,
    PB3,
    PB4,
    PB5,
    PB6,
    PB7,
    PB8,
    PB9,
    PB10,
    PB11,
    PB12,
    PB13,
    PB14,
    PB15,
    PC0,
    PC1,
    PC2,
    PC3,
    PC4,
    PC5,
    PC6,
    PC7,
    PC8,
    PC9,
    PC10,
    PC11,
    PC12,
    PC13,
    PC14,
    PC15,
    PD0,
    PD1,
    PD2,
    PD3,
    PD4,
    PD5,
    PD6,
    PD7,
    PD8,
    PD9,
    PD10,
    PD11,
    PD12,
    PD13,
    PD14,
    PD15,
    PE0,
    PE1,
    PE2,
    PE3,
    PE4,
    PE5,
    PE6,
    PE7,
    PE8,
    PE9,
    PE10,
    PE11,
    PE12,
    PE13,
    PE14,
    PE15,
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
