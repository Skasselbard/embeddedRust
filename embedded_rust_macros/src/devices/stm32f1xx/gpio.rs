use crate::{
    generation::GpioGeneration,
    types::{self, Direction, Gpio, PinMode, TriggerEdge},
};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, parse_str, Ident, Stmt, Type};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct StmGpio(
    Pin,
    Direction,
    PinMode,
    #[serde(default)] Option<TriggerEdge>,
);

impl types::Gpio for StmGpio {
    fn pin(&self) -> &dyn types::Pin {
        &self.0
    }
    fn direction(&self) -> &Direction {
        &self.1
    }
    fn mode(&self) -> &PinMode {
        &self.2
    }
    fn trigger_edge(&self) -> Option<TriggerEdge> {
        self.3
    }
    fn identifier(&self) -> Ident {
        format_ident!("{}", (self as &dyn types::Gpio).pin().name())
    }
    /// expand:
    /// ``gpio::gpiox::PXY<MODE>``
    fn ty(&self) -> Type {
        let channel_name = format_ident!("{}", self.pin().channel_name());
        let pin_type = format_ident!("{}", self.pin().to_type());
        if let PinMode::Analog = self.mode() {
            parse_str(&format!(
                "gpio::{}::{}<gpio::Analog>",
                channel_name, pin_type
            ))
            .unwrap()
        } else {
            let direction = format_ident!("{}", self.direction().to_type_string());
            let mode = format_ident!("{}", self.mode().to_type_string());
            parse_str(&format!(
                "gpio::{}::{}<gpio::{}<gpio::{}>>",
                channel_name, pin_type, direction, mode
            ))
            .unwrap()
        }
    }
    /// expand:
    /// ```
    /// let mut pin_pxy = gpiox.pxy.into_smth(&mut gpiox.control_reg);
    /// // if its an interrupt pin
    /// pin_pxy.make_interrupt_source(&mut afio);
    /// pin_pxy.trigger_on_edge(&peripherals.EXTI, Edge::EDGE_TYPE);
    /// pin_pxy.enable_interrupt(&peripherals.EXTI);
    /// ```
    fn generate(&self) -> Vec<Stmt> {
        // build identifiers
        let pin_name = format_ident!("{}", self.pin().name());
        let pin_var_ident = self.identifier();
        let channel_ident = format_ident!("{}", self.pin().channel_name());
        // the name of the gpio functions has no global pattern for all configurations
        // so we need to check the gpio configuration again
        let init_function_ident = if self.mode() == &PinMode::Analog {
            format_ident!("into_analog")
        } else {
            match self.direction() {
                Direction::Input | Direction::Output => format_ident!(
                    "into_{}_{}",
                    self.mode().to_string(),
                    self.direction().to_type_string().to_lowercase()
                ),
                Direction::Alternate => format_ident!(
                    "into_{}_{}",
                    self.direction().to_type_string().to_lowercase(),
                    self.mode().to_string(),
                ),
            }
        };
        let control_reg_ident = format_ident!("{}", control_reg(self.pin()));
        // expand: let mut pin_pxy = gpiox.pxy.into_smth(&mut gpiox.control_reg);
        let mut stmts: Vec<Stmt> = parse_quote!(
            let mut #pin_var_ident = #channel_ident.#pin_name.#init_function_ident(&mut #channel_ident.#control_reg_ident);
        );
        // if the pin shall be an interrupt source, we need additional configuration
        match self.direction() {
            Direction::Input => {
                if let Some(edge) = self.trigger_edge() {
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
}

impl StmGpio {
    pub fn new(pin: Pin, direction: Direction, mode: PinMode, edge: Option<TriggerEdge>) -> Self {
        Self(pin, direction, mode, edge)
    }
}

impl types::Pin for Pin {
    fn channel(&self) -> String {
        match self {
            pin if pin >= &Pin::PA00 && pin <= &Pin::PA15 => "a".into(),
            pin if pin >= &Pin::PB00 && pin <= &Pin::PB15 => "b".into(),
            pin if pin >= &Pin::PC00 && pin <= &Pin::PC15 => "c".into(),
            pin if pin >= &Pin::PD00 && pin <= &Pin::PD15 => "d".into(),
            pin if pin >= &Pin::PE00 && pin <= &Pin::PE15 => "e".into(),
            _ => unreachable!(),
        }
    }
    fn port(&self) -> String {
        (*self as usize % 16).to_string()
    }
    fn name(&self) -> String {
        format!("p{}{}", self.channel(), self.port())
    }
    fn channel_name(&self) -> String {
        format!("gpio{}", self.channel())
    }
    fn to_type(&self) -> String {
        self.name().to_uppercase()
    }
    fn port_constructor(&self) -> syn::Expr {
        parse_str(&format!("Port::P{:02}", (*self as usize % 16))).unwrap()
    }
    fn channel_constructor(&self) -> syn::Expr {
        parse_str(&format!("Channel::{}", self.channel().to_uppercase())).unwrap()
    }
}

pub fn control_reg(pin: &dyn crate::types::Pin) -> String {
    if (pin.port().parse::<usize>().unwrap() % 16) < 8 {
        "crl".into()
    } else {
        "crh".into()
    }
}

impl GpioGeneration for super::Generator {
    fn interrupts(&self, gpios: &Vec<Box<dyn Gpio>>) -> Vec<Stmt> {
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

/// build the statements to initialize the gpio channels
/// expand:
/// ``let mut gpiox = peripherals.GPIOX.split(&mut rcc.apb2);``
pub(crate) fn channel_init_statements(gpio_list: &Vec<Box<dyn Gpio>>) -> Vec<Stmt> {
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

/// Pin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Pin {
    #[serde(alias = "pa0", alias = "PA0")]
    PA00,
    #[serde(alias = "pa1", alias = "PA1")]
    PA01,
    #[serde(alias = "pa2", alias = "PA2")]
    PA02,
    #[serde(alias = "pa3", alias = "PA3")]
    PA03,
    #[serde(alias = "pa4", alias = "PA4")]
    PA04,
    #[serde(alias = "pa5", alias = "PA5")]
    PA05,
    #[serde(alias = "pa6", alias = "PA6")]
    PA06,
    #[serde(alias = "pa7", alias = "PA7")]
    PA07,
    #[serde(alias = "pa8", alias = "PA8")]
    PA08,
    #[serde(alias = "pa9", alias = "PA9")]
    PA09,
    #[serde(alias = "pa10", alias = "PA10")]
    PA10,
    #[serde(alias = "pa11", alias = "PA11")]
    PA11,
    #[serde(alias = "pa12", alias = "PA12")]
    PA12,
    #[serde(alias = "pa13", alias = "PA13")]
    PA13,
    #[serde(alias = "pa14", alias = "PA14")]
    PA14,
    #[serde(alias = "pa15", alias = "PA15")]
    PA15,
    #[serde(alias = "pb0", alias = "PB0")]
    PB00,
    #[serde(alias = "pb1", alias = "PB1")]
    PB01,
    #[serde(alias = "pb2", alias = "PB2")]
    PB02,
    #[serde(alias = "pb3", alias = "PB3")]
    PB03,
    #[serde(alias = "pb4", alias = "PB4")]
    PB04,
    #[serde(alias = "pb5", alias = "PB5")]
    PB05,
    #[serde(alias = "pb6", alias = "PB6")]
    PB06,
    #[serde(alias = "pb7", alias = "PB7")]
    PB07,
    #[serde(alias = "pb8", alias = "PB8")]
    PB08,
    #[serde(alias = "pb9", alias = "PB9")]
    PB09,
    #[serde(alias = "pb10", alias = "PB10")]
    PB10,
    #[serde(alias = "pb11", alias = "PB11")]
    PB11,
    #[serde(alias = "pb12", alias = "PB12")]
    PB12,
    #[serde(alias = "pb13", alias = "PB13")]
    PB13,
    #[serde(alias = "pb14", alias = "PB14")]
    PB14,
    #[serde(alias = "pb15", alias = "PB15")]
    PB15,
    #[serde(alias = "pc0", alias = "PC0")]
    PC00,
    #[serde(alias = "pc1", alias = "PC1")]
    PC01,
    #[serde(alias = "pc2", alias = "PC2")]
    PC02,
    #[serde(alias = "pc3", alias = "PC3")]
    PC03,
    #[serde(alias = "pc4", alias = "PC4")]
    PC04,
    #[serde(alias = "pc5", alias = "PC5")]
    PC05,
    #[serde(alias = "pc6", alias = "PC6")]
    PC06,
    #[serde(alias = "pc7", alias = "PC7")]
    PC07,
    #[serde(alias = "pc8", alias = "PC8")]
    PC08,
    #[serde(alias = "pc9", alias = "PC9")]
    PC09,
    #[serde(alias = "pc10", alias = "PC10")]
    PC10,
    #[serde(alias = "pc11", alias = "PC11")]
    PC11,
    #[serde(alias = "pc12", alias = "PC12")]
    PC12,
    #[serde(alias = "pc13", alias = "PC13")]
    PC13,
    #[serde(alias = "pc14", alias = "PC14")]
    PC14,
    #[serde(alias = "pc15", alias = "PC15")]
    PC15,
    #[serde(alias = "pd0", alias = "PD0")]
    PD00,
    #[serde(alias = "pd1", alias = "PD1")]
    PD01,
    #[serde(alias = "pd2", alias = "PD2")]
    PD02,
    #[serde(alias = "pd3", alias = "PD3")]
    PD03,
    #[serde(alias = "pd4", alias = "PD4")]
    PD04,
    #[serde(alias = "pd5", alias = "PD5")]
    PD05,
    #[serde(alias = "pd6", alias = "PD6")]
    PD06,
    #[serde(alias = "pd7", alias = "PD7")]
    PD07,
    #[serde(alias = "pd8", alias = "PD8")]
    PD08,
    #[serde(alias = "pd9", alias = "PD9")]
    PD09,
    #[serde(alias = "pd10", alias = "PD10")]
    PD10,
    #[serde(alias = "pd11", alias = "PD11")]
    PD11,
    #[serde(alias = "pd12", alias = "PD12")]
    PD12,
    #[serde(alias = "pd13", alias = "PD13")]
    PD13,
    #[serde(alias = "pd14", alias = "PD14")]
    PD14,
    #[serde(alias = "pd15", alias = "PD15")]
    PD15,
    #[serde(alias = "pe0", alias = "PE0")]
    PE00,
    #[serde(alias = "pe1", alias = "PE1")]
    PE01,
    #[serde(alias = "pe2", alias = "PE2")]
    PE02,
    #[serde(alias = "pe3", alias = "PE3")]
    PE03,
    #[serde(alias = "pe4", alias = "PE4")]
    PE04,
    #[serde(alias = "pe5", alias = "PE5")]
    PE05,
    #[serde(alias = "pe6", alias = "PE6")]
    PE06,
    #[serde(alias = "pe7", alias = "PE7")]
    PE07,
    #[serde(alias = "pe8", alias = "PE8")]
    PE08,
    #[serde(alias = "pe9", alias = "PE9")]
    PE09,
    #[serde(alias = "pe0", alias = "PE0")]
    PE10,
    #[serde(alias = "pe1", alias = "PE1")]
    PE11,
    #[serde(alias = "pe2", alias = "PE2")]
    PE12,
    #[serde(alias = "pe3", alias = "PE3")]
    PE13,
    #[serde(alias = "pe4", alias = "PE4")]
    PE14,
    #[serde(alias = "pe5", alias = "PE5")]
    PE15,
}
