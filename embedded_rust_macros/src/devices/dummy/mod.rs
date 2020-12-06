//! This is a minimal example implementation for a device.
//! Each device has to be added to the types::Config variants and
//! the acces functions of the Config enum (the compiler will complain^^).
//!
//! Look at the trait documentation for more information what the functions are used for.
use crate::types::Serial;
use crate::types::{self, Direction, Gpio, Pin, PinMode, TriggerEdge};
use crate::{generation::*, types::Baud};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::parse_str;
use types::{PWMInterface, UnitHz};

/// The Generator struct is used to introduce all code generation functions.
/// It has to implement the Generator trait.
pub struct DummyGenerator;
impl Generator for DummyGenerator {}

/// A tuple enum is a good way to deserialize a struct.
/// This way the json boilerplate is kept low.
/// It is asumed that every device has its own pin type (covered with the
/// types::Pin trait). A direction (in/out) and interrupt triger edge is
/// general enough to be a general type. Until it proves to be useful the pin mode
/// is also a general type
///
/// Use the #[serde(default)] annotation to make the interrupt trigger edge optional
/// to parse (so that it can be omitted)
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct DummyGpio(
    DummyPin,
    Direction,
    PinMode,
    #[serde(default)] Option<TriggerEdge>,
);

/// Pins should match the naming conventions of the device
/// E.g. Arduino Nano and stm32f1 pins are named pxy,
/// where x is a channel and y is a number (called "port" in this project)
/// which is repeated for each channel.
///
/// For a compact Deserialization it is advisable to make this an enum variant
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum DummyPin {
    // You can add aliases for deserialization
    // To make the definition user frfendly you
    // can add different variants for case sensitivity
    #[serde(alias = "Eins", alias = "Uno")]
    One,
    #[serde(alias = "Zwei", alias = "Dos")]
    Two,
    #[serde(alias = "Drei", alias = "Tres")]
    Three,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DummyPWM {
    pins: Vec<DummyPin>,
    frequency: (u32, UnitHz),
}
#[derive(Clone, Debug, Deserialize)]
pub struct DummySerial {
    #[serde(alias = "rx", alias = "receive")]
    receive_pin: DummyPin,
    #[serde(alias = "tx", alias = "transmit")]
    transmit_pin: DummyPin,
    #[serde(alias = "speed", alias = "baud")]
    baudrate: Baud,
}

impl DeviceGeneration for DummyGenerator {
    fn generate_imports(&self) -> std::vec::Vec<syn::Stmt> {
        syn::parse_quote!(
            use core::usize;
        )
    }
    fn generate_device_init(&self) -> std::vec::Vec<syn::Stmt> {
        vec![]
    }
    fn generate_channels(
        &self,
        _: &std::vec::Vec<Box<dyn types::Gpio>>,
    ) -> std::vec::Vec<syn::Stmt> {
        vec![]
    }
}

impl SysGeneration for DummyGenerator {
    fn generate_clock(
        &self,
        _: &std::option::Option<types::Frequency>,
    ) -> std::vec::Vec<syn::Stmt> {
        vec![]
    }
}

impl GpioGeneration for DummyGenerator {
    fn interrupts(&self, _: &std::vec::Vec<Box<dyn types::Gpio>>) -> std::vec::Vec<syn::Stmt> {
        //TODO:
        vec![]
    }
}

impl PWMGeneration for DummyGenerator {}

impl SerialGeneration for DummyGenerator {
    fn generate_serials(&self, _serials: &Vec<&dyn Serial>) -> Vec<syn::Stmt> {
        // TODO:
        todo!()
    }
}

// This implementation should already fit most perposes if you
// held on to the DummyGpio tuple struct example
impl Gpio for DummyGpio {
    fn pin(&self) -> &dyn types::Pin {
        &self.0
    }
    fn direction(&self) -> &types::Direction {
        &self.1
    }
    fn mode(&self) -> &types::PinMode {
        &self.2
    }
    fn trigger_edge(&self) -> std::option::Option<types::TriggerEdge> {
        self.3
    }
    fn identifier(&self) -> syn::Ident {
        format_ident!("{}", (self as &dyn types::Gpio).pin().name())
    }
    fn ty(&self) -> syn::Type {
        syn::parse_str(&format!("()")).unwrap()
    }

    fn generate(&self) -> Vec<syn::Stmt> {
        let pin_ident = self.identifier();
        syn::parse_quote!(
            let #pin_ident = ();
        )
    }
}

impl Pin for DummyPin {
    fn channel(&self) -> std::string::String {
        "".into()
    }
    fn port(&self) -> std::string::String {
        (*self as usize).to_string()
    }
    fn name(&self) -> std::string::String {
        format!("pin{}{}", self.channel(), self.port())
    }
    fn channel_name(&self) -> std::string::String {
        format!("{}", self.channel())
    }
    fn to_type(&self) -> std::string::String {
        self.name().to_uppercase()
    }
    fn port_constructor(&self) -> syn::Expr {
        parse_str(&format!("Port::P{:02}", (*self as usize))).unwrap()
    }
    fn channel_constructor(&self) -> syn::Expr {
        parse_str(&format!("Channel::A")).unwrap()
    }
}

impl PWMInterface for DummyPWM {
    fn pins(&self) -> Vec<&dyn Pin> {
        self.pins.iter().map(|pin| pin as &dyn Pin).collect()
    }

    fn tys(&self) -> Vec<syn::Type> {
        todo!()
    }

    fn frequency(&self) -> types::Frequency {
        types::Frequency::from(&self.frequency)
    }

    fn generate(&self) -> Vec<syn::Stmt> {
        vec![]
    }

    fn pins_as_gpios(&self) -> Vec<Box<dyn Gpio>> {
        todo!()
    }
}

impl Serial for DummySerial {
    fn receive_pin(&self) -> &dyn crate::types::Pin {
        &self.receive_pin
    }

    fn transmit_pin(&self) -> &dyn crate::types::Pin {
        &self.transmit_pin
    }

    fn baud(&self) -> Baud {
        self.baudrate
    }

    fn name(&self) -> String {
        todo!()
    }

    fn generate(&self) -> Vec<syn::Stmt> {
        todo!()
    }

    fn pins_as_gpio(&self) -> Vec<Box<dyn Gpio>> {
        todo!()
    }

    fn word_ty(&self) -> syn::Type {
        todo!()
    }

    fn serial_id(&self) -> String {
        todo!()
    }

    fn generate_enable_interrupt(&self) -> syn::Stmt {
        todo!()
    }

    fn tx_ty(&self) -> syn::Type {
        todo!()
    }

    fn read_err_ty(&self) -> syn::Type {
        todo!()
    }
}
