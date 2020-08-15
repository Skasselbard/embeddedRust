pub mod gpio;

pub use self::gpio::*;
use crate::generation::{self, DeviceGeneration, GpioGeneration, SysGeneration};
use crate::types::{Direction, Frequency, Gpio, PinMode, TriggerEdge};
use quote::format_ident;
use syn::{parse_quote, parse_str, Ident, Stmt, Type};

macro_rules! peripherals_ident {
    () => {
        format_ident!("peripherals")
    };
}

pub struct Generator;

impl generation::Generator for Generator {}

impl DeviceGeneration for Generator {
    fn generate_imports(&self) -> std::vec::Vec<syn::Stmt> {
        parse_quote!(
            use stm32f1xx_hal::gpio::{Edge, ExtiPin};
            use stm32f1xx_hal::pac;
            use stm32f1xx_hal::prelude::*;
            use embedded_rust::device::{InputPin, OutputPin};
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
    fn generate_channels(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt> {
        channel_init_statements(gpios)
    }
    fn generate_gpios(&self, gpios: &Vec<&dyn Gpio>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for gpio in gpios {
            stmts.append(&mut generate_gpio(*gpio));
        }
        stmts
    }
    fn input_error(&self) -> Type {
        parse_str("core::convert::Infallible").unwrap()
    }
    fn output_error(&self) -> Type {
        parse_str("core::convert::Infallible").unwrap()
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

pub fn control_reg(pin: &dyn crate::types::Pin) -> String {
    if (pin.port().parse::<usize>().unwrap() % 16) < 8 {
        "crl".into()
    } else {
        "crh".into()
    }
}
/// expand:
/// ``stm32f1xx_hal::gpio::gpiox::PXY<MODE>``
pub(crate) fn gpio_to_type(gpio: &dyn Gpio) -> Type {
    let channel_name = format_ident!("{}", gpio.pin().channel_name());
    let pin_type = format_ident!("{}", gpio.pin().to_type());
    if let PinMode::Analog = gpio.mode() {
        parse_str(&format!(
            "stm32f1xx_hal::gpio::{}::{}<stm32f1xx_hal::gpio::Analog>",
            channel_name, pin_type
        ))
        .unwrap()
    } else {
        let direction = format_ident!("{}", gpio.direction().to_type_string());
        let mode = format_ident!("{}", gpio.mode().to_type_string());
        parse_str(&format!(
            "stm32f1xx_hal::gpio::{}::{}<stm32f1xx_hal::gpio::{}<stm32f1xx_hal::gpio::{}>>",
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
