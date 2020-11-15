macro_rules! peripherals_ident {
    () => {
        format_ident!("peripherals")
    };
}

pub mod gpio;
mod pwm;
mod serial;

pub use self::gpio::*;
pub use self::pwm::*;
pub use self::serial::*;
use crate::generation::{self, DeviceGeneration, SysGeneration};
use crate::types::{Frequency, Gpio};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Timer {
    #[serde(alias = "tim1", alias = "TIM1")]
    Tim1,
    #[serde(alias = "tim2", alias = "TIM2")]
    Tim2,
    #[serde(alias = "tim3", alias = "TIM3")]
    Tim3,
    #[serde(alias = "tim4", alias = "TIM4")]
    Tim4,
    // #[serde(alias = "tim5", alias = "TIM5")]
    // Tim5,
    // #[serde(alias = "tim6", alias = "TIM6")]
    // Tim6,
    // #[serde(alias = "tim7", alias = "TIM7")]
    // Tim7,
    // #[serde(alias = "tim8", alias = "TIM8")]
    // Tim8,
    // #[serde(alias = "tim9", alias = "TIM9")]
    // Tim9,
    // #[serde(alias = "tim10", alias = "TIM10")]
    // Tim10,
    // #[serde(alias = "tim11", alias = "TIM11")]
    // Tim11,
    // #[serde(alias = "tim12", alias = "TIM12")]
    // Tim12,
    // #[serde(alias = "tim13", alias = "TIM13")]
    // Tim13,
    // #[serde(alias = "tim14", alias = "TIM14")]
    // Tim14,
}

impl Timer {
    pub fn name(&self) -> String {
        match self {
            Timer::Tim1 => "tim1".into(),
            Timer::Tim2 => "tim2".into(),
            Timer::Tim3 => "tim3".into(),
            Timer::Tim4 => "tim4".into(),
        }
    }
    pub fn remap(&self, _pins: &Vec<Pin>) -> String {
        // FIXME: calculate remap type wit pins https://docs.rs/stm32f1xx-hal/0.6.1/stm32f1xx_hal/timer/index.html
        match self {
            Timer::Tim1 => "Tim1NoRemap".into(),
            Timer::Tim2 => "Tim2NoRemap".into(),
            Timer::Tim3 => "Tim3NoRemap".into(),
            Timer::Tim4 => "Tim4NoRemap".into(),
        }
    }
    pub fn peripheral_bus(&self) -> String {
        match self {
            Timer::Tim1 => "apb2".into(),
            Timer::Tim2 => "apb1".into(),
            Timer::Tim3 => "apb1".into(),
            Timer::Tim4 => panic!("tim 4 does not map on peripheral bus"),
        }
    }

    pub fn channel(&self, pin: &Pin) -> String {
        match (self, pin) {
            (Timer::Tim1, Pin::PA08) => "C1",
            (Timer::Tim1, Pin::PA09) => "C2",
            (Timer::Tim1, Pin::PA10) => "C3",
            (Timer::Tim1, Pin::PA11) => "C4",
            (Timer::Tim1, Pin::PE09) => "C1",
            (Timer::Tim1, Pin::PE11) => "C2",
            (Timer::Tim1, Pin::PE13) => "C3",
            (Timer::Tim1, Pin::PE14) => "C4",
            (Timer::Tim2, Pin::PA00) => "C1",
            (Timer::Tim2, Pin::PA01) => "C2",
            (Timer::Tim2, Pin::PA02) => "C3",
            (Timer::Tim2, Pin::PA03) => "C4",
            (Timer::Tim2, Pin::PA15) => "C1",
            (Timer::Tim2, Pin::PB03) => "C2",
            (Timer::Tim2, Pin::PB10) => "C3",
            (Timer::Tim2, Pin::PB11) => "C4",
            (Timer::Tim3, Pin::PA06) => "C1",
            (Timer::Tim3, Pin::PA07) => "C2",
            (Timer::Tim3, Pin::PB00) => "C3",
            (Timer::Tim3, Pin::PB01) => "C4",
            (Timer::Tim3, Pin::PB04) => "C1",
            (Timer::Tim3, Pin::PB05) => "C2",
            (Timer::Tim3, Pin::PC06) => "C1",
            (Timer::Tim3, Pin::PC07) => "C2",
            (Timer::Tim3, Pin::PC08) => "C3",
            (Timer::Tim3, Pin::PC09) => "C4",
            (Timer::Tim4, Pin::PC06) => "C1",
            (Timer::Tim4, Pin::PC07) => "C2",
            (Timer::Tim4, Pin::PC08) => "C3",
            (Timer::Tim4, Pin::PC09) => "C4",
            (Timer::Tim4, Pin::PD12) => "C1",
            (Timer::Tim4, Pin::PD13) => "C2",
            (Timer::Tim4, Pin::PD14) => "C3",
            (Timer::Tim4, Pin::PD15) => "C4",
            _ => panic!("pin does not map on timer channel"),
        }
        .into()
    }
}

pub struct Generator;

impl generation::Generator for Generator {}

impl DeviceGeneration for Generator {
    fn generate_imports(&self) -> std::vec::Vec<syn::Stmt> {
        parse_quote!(
            use stm32f1xx_hal::prelude::*;
            use stm32f1xx_hal::gpio::{self, Edge, ExtiPin};
            use stm32f1xx_hal::timer::{self, Timer};
            use stm32f1xx_hal::pwm::{self, PwmChannel};
            use stm32f1xx_hal::pac;
            use stm32f1xx_hal::serial::{self, Config};
            use embedded_rust::resources::{Resource, Pin, InputPin, OutputPin, PWMPin, Serial};
            use embedded_rust::device::{Port, Channel, SerialID};
            use embedded_rust::Runtime;
        )
    }
    fn generate_device_init(&self) -> std::vec::Vec<syn::Stmt> {
        let peripherals_ident = peripherals_ident!();
        parse_quote!(
            let #peripherals_ident = stm32f1xx_hal::pac::Peripherals::take().unwrap();
            let mut flash = #peripherals_ident.FLASH.constrain();
        )
    }
    fn generate_channels(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt> {
        channel_init_statements(gpios)
    }
}

impl SysGeneration for Generator {
    fn generate_clock(&self, sys_freq: &Option<Frequency>) -> Vec<Stmt> {
        let peripherals_ident = peripherals_ident!();
        let sys_freq: Option<Stmt> = sys_freq.as_ref().map(|f| {
            let f = f.0;
            parse_quote!(let cfgr = cfgr.sysclk(#f.hz());)
        });
        let clock = parse_quote!(
            let mut rcc = #peripherals_ident.RCC.constrain();
            let cfgr = rcc.cfgr;
            #sys_freq
            let clocks = cfgr.freeze(&mut flash.acr);
            let mut afio = #peripherals_ident.AFIO.constrain(&mut rcc.apb2);
        );
        clock
    }
}
