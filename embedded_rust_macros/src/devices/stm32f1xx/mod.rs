macro_rules! peripherals_ident {
    () => {
        format_ident!("peripherals")
    };
}

pub mod gpio;
mod pwm;

pub use self::gpio::*;
pub use self::pwm::*;
use crate::generation::{self, DeviceGeneration, GpioGeneration, SysGeneration};
use crate::types::{Direction, Frequency, Gpio, PinMode, TriggerEdge};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, parse_str, Stmt, Type};

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
            use embedded_rust::resources::{Pin, InputPin, OutputPin, PWMPin};
            use embedded_rust::device::{Port, Channel};
            use embedded_rust::resources::{Resource};
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
    fn generate_channels(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt> {
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

impl GpioGeneration for Generator {
    fn generate_gpios(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for gpio in gpios {
            stmts.append(&mut generate_gpio(*gpio));
        }
        stmts
    }
    fn interrupts(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for gpio in gpios {
            if gpio.trigger_edge().is_some() {
                let interrupt_line = format_ident!(
                    "EXTI{}",
                    match gpio.pin().port().as_str() {
                        "0" => "0",
                        "1" => "1",
                        "2" => "2",
                        "3" => "3",
                        "4" => "4",
                        "5" | "6" | "7" | "8" | "9" => "9_5",
                        "10" | "11" | "12" | "13" | "14" | "15" => "15_10",
                        _ => unreachable!(),
                    }
                );
                stmts.push(parse_quote!(
                    stm32f1xx_hal::pac::NVIC::unmask(
                        stm32f1xx_hal::pac::Interrupt::#interrupt_line
                    );
                ));
            }
        }
        stmts
    }
}

/// expand:
/// ``gpio::gpiox::PXY<MODE>``
pub(crate) fn gpio_to_type(gpio: &dyn Gpio) -> Type {
    let channel_name = format_ident!("{}", gpio.pin().channel_name());
    let pin_type = format_ident!("{}", gpio.pin().to_type());
    if let PinMode::Analog = gpio.mode() {
        parse_str(&format!(
            "gpio::{}::{}<gpio::Analog>",
            channel_name, pin_type
        ))
        .unwrap()
    } else {
        let direction = format_ident!("{}", gpio.direction().to_type_string());
        let mode = format_ident!("{}", gpio.mode().to_type_string());
        parse_str(&format!(
            "gpio::{}::{}<gpio::{}<gpio::{}>>",
            channel_name, pin_type, direction, mode
        ))
        .unwrap()
    }
}

/// build the statements to initialize the gpio channels
/// expand:
/// ``let mut gpiox = peripherals.GPIOX.split(&mut rcc.apb2);``
pub(crate) fn channel_init_statements(gpio_list: &Vec<&dyn Gpio>) -> Vec<Stmt> {
    use std::collections::HashSet;
    let mut channels = HashSet::with_capacity(5);
    let mut stmts = Vec::with_capacity(5);
    for gpio in gpio_list {
        // only one initialization for each channel
        if !channels.contains(&gpio.pin().channel()) {
            // remember initialized channel
            channels.insert(gpio.pin().channel());
            let channel_lower = format_ident!("{}", gpio.pin().channel_name());
            let channel_upper = format_ident!("{}", gpio.pin().channel_name().to_uppercase());
            let peripherals_ident = peripherals_ident!();
            // expand: let mut gpiox = peripherals.GPIOX.split(&mut rcc.apb2);
            // its always apb2 on this boards
            stmts.push(parse_quote!(
                let mut #channel_lower = #peripherals_ident.#channel_upper.split(&mut rcc.apb2);
            ))
        }
    }
    stmts
}

/// expand:
/// ```
/// let mut pin_pxy = gpiox.pxy.into_smth(&mut gpiox.control_reg);
/// // if its an interrupt pin
/// pin_pxy.make_interrupt_source(&mut afio);
/// pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
/// pin_pxy.enable_interrupt(&peripherals.EXTI);
/// ```
pub(crate) fn generate_gpio(gpio: &dyn Gpio) -> Vec<Stmt> {
    // build identifiers
    let pin_name = format_ident!("{}", gpio.pin().name());
    let pin_var_ident = gpio.identifier();
    let channel_ident = format_ident!("{}", gpio.pin().channel_name());
    // the name of the gpio functions has no global pattern for all configurations
    // so we need to check the gpio configuration again
    let init_function_ident = if gpio.mode() == &PinMode::Analog {
        format_ident!("into_analog")
    } else {
        format_ident!(
            "into_{}_{}",
            gpio.mode().to_string(),
            gpio.direction().to_type_string().to_lowercase()
        )
    };
    let control_reg_ident = format_ident!("{}", control_reg(gpio.pin()));
    // expand: let mut pin_pxy = gpiox.pxy.into_smth(&mut gpiox.control_reg);
    let mut stmts: Vec<Stmt> = parse_quote!(
        let mut #pin_var_ident = #channel_ident.#pin_name.#init_function_ident(&mut #channel_ident.#control_reg_ident);
    );
    // if the pin shall be an interrupt source, we need additional configuration
    match gpio.direction() {
        Direction::Input => {
            if let Some(edge) = gpio.trigger_edge() {
                let peripherals_ident = peripherals_ident!();
                let edge_ident = match edge {
                    TriggerEdge::Rising => format_ident!("RISING"),
                    TriggerEdge::Falling => format_ident!("FALLING"),
                    TriggerEdge::All => format_ident!("RISING_FALLING"),
                };
                // expand:
                // pin_pxy.make_interrupt_source(&mut afio);
                // pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
                // pin_pxy.enable_interrupt(&peripherals.EXTI);
                let mut interrupt_stmts: Vec<Stmt> = parse_quote!(
                    #pin_var_ident.make_interrupt_source(&mut afio);
                    #pin_var_ident.trigger_on_edge(&#peripherals_ident.EXTI, Edge::#edge_ident);
                    #pin_var_ident.enable_interrupt(&#peripherals_ident.EXTI);
                );
                stmts.append(&mut interrupt_stmts);
            }
        }
        _ => {}
    }
    stmts
}
