use super::{Generator, Pin, StmGpio};
use crate::{
    generation::SerialGeneration,
    types::{Baud, Direction, PinMode, Serial},
};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, parse_str, Stmt, Type};

impl SerialGeneration for Generator {}

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
    fn name(&self) -> String {
        self.name()
    }
    fn pins_as_gpio(&self) -> Vec<Box<dyn crate::types::Gpio>> {
        vec![self.reveceive_as_gpio(), self.transmit_as_gpio()]
    }
    fn rx_ty(&self) -> Type {
        let name = format_ident!("{}", self.name().to_uppercase());
        parse_quote!(
            serial::Rx<pac::#name>
        )
    }
    fn tx_ty(&self) -> Type {
        let name = format_ident!("{}", self.name().to_uppercase());
        parse_quote!(
            serial::Tx<pac::#name>
        )
    }
    fn read_err_ty(&self) -> Type {
        parse_quote!(serial::Error)
    }
    fn generate(&self) -> Vec<Stmt> {
        let name = format_ident!("{}", self.name());
        let name_upper = format_ident!("{}", self.name().to_uppercase());
        // Pins used for tx and rx
        let tx_pin = format_ident!("{}", self.transmit_pin().name());
        let rx_pin = format_ident!("{}", self.receive_pin().name());
        // RX and TX channel after splitting the serial
        let tx_channel = format_ident!("{}_tx", self.name());
        let rx_channel = format_ident!("{}_rx", self.name());
        let peripherals = peripherals_ident!();
        let baud = self.baud().0;
        let ap_bus = match self.name().as_str() {
            "usart1" => format_ident!("{}", "apb2"),
            "usart2" => format_ident!("{}", "apb1"),
            "usart3" => format_ident!("{}", "apb1"),
            _ => unreachable!(),
        };
        parse_quote!(
            let mut #name = serial::Serial::#name(
                #peripherals.#name_upper,
                (#tx_pin, #rx_pin),
                &mut afio.mapr,
                Config::default().baudrate(#baud.bps()),
                clocks,
                &mut rcc.#ap_bus
            );
            let (mut #tx_channel, mut #rx_channel) = #name.split();
            #rx_channel.listen();
        )
    }

    fn generate_enable_interrupt(&self) -> Stmt {
        let interrupt_line = format_ident!("{}", self.name().to_uppercase());
        parse_quote!(
            stm32f1xx_hal::pac::NVIC::unmask(
                stm32f1xx_hal::pac::Interrupt::#interrupt_line
            );
        )
    }

    fn word_ty(&self) -> Type {
        parse_str("u8").unwrap()
    }

    fn serial_id(&self) -> String {
        match self {
            StmSerial::USART1(_) => "Usart1".into(),
            StmSerial::USART2(_) => "Usart2".into(),
            StmSerial::USART3(_) => "Usart3".into(),
        }
    }
}
