use super::{Generator, Pin};
use crate::types::{Baud, SerialInterface};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, parse_str, Stmt};

/// ```
/// "Serial":[{
///     "rx":     "PA1",
///     "tx":     "PA1",
///     "baudrate": "9600"
/// }]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Serial {
    #[serde(alias = "rx", alias = "receive")]
    receive_pin: Pin,
    #[serde(alias = "tx", alias = "transmit")]
    transmit_pin: Pin,
    #[serde(alias = "speed", alias = "baud")]
    baudrate: Baud,
    //TODO: rest of stm32f1xx_hal::serial::Config
}

impl SerialInterface for Serial {
    fn receive_pin(&self) -> &dyn crate::types::Pin {
        &self.receive_pin
    }

    fn transmit_pin(&self) -> &dyn crate::types::Pin {
        &self.transmit_pin
    }

    fn baud(&self) -> Baud {
        self.baudrate
    }
}
