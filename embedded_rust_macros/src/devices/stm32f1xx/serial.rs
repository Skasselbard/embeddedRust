use super::{Generator, Pin, StmGpio};
use crate::{
    generation::SerialGeneration,
    types::{Baud, Direction, PinMode, Serial},
};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{Stmt, Type, parse_quote, parse_str};

/// ```
/// "Serial":[
///     {"usart1":{
///         "rx": "PA2",
///         "tx": "PA3",
///         "baud": 9600
///     }}
/// ]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub enum StmSerial {
    #[serde(alias = "usart1", alias = "Usart1")]
    USART1(SerialInner),
    #[serde(alias = "usart2", alias = "Usart2")]
    USART2(SerialInner),
    #[serde(alias = "usart3", alias = "Usart3")]
    USART3(SerialInner),
    // #[serde(alias = "usart4", alias = "Usart4")]
    // USART4(SerialInner),
    // #[serde(alias = "usart5", alias = "Usart5")]
    // USART5(SerialInner),
}
#[derive(Debug, Clone, Deserialize)]
pub struct SerialInner {
    #[serde(alias = "rx", alias = "receive")]
    pub receive_pin: Pin,
    #[serde(alias = "tx", alias = "transmit")]
    pub transmit_pin: Pin,
    #[serde(alias = "speed", alias = "baud")]
    pub baudrate: Baud,
    //TODO: rest of stm32f1xx_hal::serial::Config
}

impl StmSerial {
    fn inner(&self) -> &SerialInner {
        match self {
            StmSerial::USART1(i) => i,
            StmSerial::USART2(i) => i,
            StmSerial::USART3(i) => i,
            // StmSerial::USART4(i) => i,
            // StmSerial::USART5(i) => i,
        }
    }
    fn name(&self) -> String {
        match self {
            StmSerial::USART1(_) => String::from("usart1"),
            StmSerial::USART2(_) => String::from("usart2"),
            StmSerial::USART3(_) => String::from("usart3"),
            // StmSerial::USART4(_) => String::from("usart4"),
            // StmSerial::USART5(_) => String::from("usart5"),
        }
    }
}

impl Serial for StmSerial {
    fn receive_pin(&self) -> &dyn crate::types::Pin {
        &self.inner().receive_pin
    }

    fn transmit_pin(&self) -> &dyn crate::types::Pin {
        &self.inner().transmit_pin
    }

    fn baud(&self) -> Baud {
        self.inner().baudrate
    }

    fn reveceive_as_gpio(&self) -> Box<dyn crate::types::Gpio> {
        Box::new(StmGpio::new(
            self.inner().receive_pin,
            Direction::Input,
            PinMode::Floating,
            None, //TODO
        ))
    }

    fn transmit_as_gpio(&self) -> Box<dyn crate::types::Gpio> {
        Box::new(StmGpio::new(
            self.inner().transmit_pin,
            Direction::Alternate,
            PinMode::PushPull,
            None, //TODO
        ))
    }

    fn name(&self) -> String {
        self.name()
    }

    fn ty(&self) -> Type{
        parse_str(&format!(
            "Serial<{},<({}, {})>>",
            self.name().to_uppercase(),
            self.transmit_pin().to_type(),
            self.receive_pin().to_type(),
        )).unwrap()
    }
}

impl SerialGeneration for super::Generator {
    fn pins_as_gpio(&self, serials: &Vec<&dyn Serial>) -> Vec<Box<dyn crate::types::Gpio>> {
        let mut gpios = vec![];
        for serial in serials {
            gpios.push(serial.reveceive_as_gpio());
            gpios.push(serial.transmit_as_gpio());
        }
        gpios
    }
    fn generate_serials(&self, serials: &Vec<&dyn Serial>) -> Vec<Stmt> {
        let mut stmts = vec![];
        for serial in serials {
            let name = format_ident!("{}", serial.name());
            let name_upper = format_ident!("{}", serial.name().to_uppercase());
            let tx = format_ident!("{}", serial.transmit_pin().name());
            let rx = format_ident!("{}", serial.receive_pin().name());
            let peripherals = peripherals_ident!();
            let baud = serial.baud().0;
            let ap_bus = match serial.name().as_str() {
                "usart1" => format_ident!("{}", "apb2"),
                "usart2" => format_ident!("{}", "apb1"),
                "usart3" => format_ident!("{}", "apb1"),
                _ => unreachable!(),
            };
            stmts.append(&mut parse_quote!(
                let mut #name = Serial::#name(
                    #peripherals.#name_upper,
                    (#tx, #rx),
                    &mut afio.mapr,
                    Config::default().baudrate(#baud.bps()),
                    clocks,
                    &mut rcc.#ap_bus
                );
            ))
        }
        stmts
    }
}
