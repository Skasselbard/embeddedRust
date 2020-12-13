use serde_derive::Deserialize;

use syn::{Expr, Ident, Type};

/// Gpios are device dependend, so they have some general behavior that they should provide.
/// Currently it is expected that each gpio has a pin (covert with
/// an additional trait), a direction (in/out), an interrupt trigger edge configuration
/// and a mode. It is useful to structure the actual gpio type in a similar way (see
/// dummy example).
pub trait Gpio {
    fn pin(&self) -> &dyn Pin;
    fn direction(&self) -> &Direction;
    fn mode(&self) -> &PinMode;
    fn trigger_edge(&self) -> Option<TriggerEdge>;
    // TODO: this might be useful for all traits
    fn identifier(&self) -> Ident;
    fn ty(&self) -> Type;
    fn generate(&self) -> Vec<syn::Stmt>;
}

pub trait PWMInterface {
    fn pins(&self) -> Vec<&dyn Pin>;
    fn pins_as_gpios(&self) -> Vec<Box<dyn Gpio>>;
    fn tys(&self) -> Vec<Type>;
    fn frequency(&self) -> Frequency;
    fn generate(&self) -> Vec<syn::Stmt>;
}

pub trait Serial {
    fn name(&self) -> String;
    fn serial_id(&self) -> String;
    fn receive_pin(&self) -> &dyn Pin;
    fn transmit_pin(&self) -> &dyn Pin;
    fn pins_as_gpio(&self) -> Vec<Box<dyn crate::types::Gpio>>;
    fn rx_ty(&self) -> Type;
    fn tx_ty(&self) -> Type;
    fn read_err_ty(&self) -> Type;
    fn word_ty(&self) -> Type;
    fn baud(&self) -> Baud;
    fn generate(&self) -> Vec<syn::Stmt>;
    // TODO: probably not necessary
    fn generate_enable_interrupt(&self) -> syn::Stmt;
}

/// The trait that each device pin should implement. For a complex example impression
/// look at the Pin implementation of the stm32f1xx::device.
pub trait Pin {
    /// Each channel has the same ports.
    fn channel(&self) -> String;
    /// The port of a channel. Probably a number.
    fn port(&self) -> String;
    /// Generate the construction expression for the pin-port
    fn port_constructor(&self) -> Expr;
    /// Can be used to build identifiers;
    fn name(&self) -> String;
    /// A complete name of the pin channel. 'gpioa' - 'gpioe' in the
    /// stm32_hal.
    fn channel_name(&self) -> String;
    /// Generate the construction expression for the pinchannel-
    fn channel_constructor(&self) -> Expr;
    /// In the stm32_hal, each pin has a different typ of the form
    /// Pin<Mode> (e.g. PA0<Alternate<PushPull>> or PB4<Analog>)
    /// This function should return the 'Pin' part of 'Pin<Mode>
    /// so that the complete type can be build in Gpio::ty function.
    fn to_type(&self) -> String;
}

#[derive(Deserialize, Debug)]
pub struct Sys {
    pub sys_clock: Option<(u32, UnitHz)>,
    heap_size: (usize, UnitByte),
    pub log: Option<Log>,
}

impl Sys {
    pub fn heap_size(&self) -> usize {
        match self.heap_size.1 {
            UnitByte::Byte => self.heap_size.0,
            UnitByte::KB => self.heap_size.0 * 1024,
            UnitByte::MB => self.heap_size.0 * 1024 * 1024,
            UnitByte::GB => self.heap_size.0 * 1024 * 1024,
        }
    }
    pub fn sys_clock(&self) -> Option<usize> {
        self.sys_clock
            .as_ref()
            .map(|c| Frequency::from(c).0 as usize)
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum UnitHz {
    #[serde(alias = "hz", alias = "Hz")]
    Hz,
    #[serde(alias = "k", alias = "K", alias = "khz", alias = "KHz")]
    KHz,
    #[serde(alias = "m", alias = "M", alias = "mhz", alias = "MHz")]
    MHz,
    #[serde(alias = "g", alias = "G", alias = "ghz", alias = "GHz")]
    GHz,
}

#[derive(Deserialize, Debug)]
pub enum UnitByte {
    #[serde(alias = "byte", alias = "b")]
    Byte,
    #[serde(alias = "k", alias = "K", alias = "kb", alias = "KB")]
    KB,
    #[serde(alias = "m", alias = "M", alias = "mb", alias = "MB")]
    MB,
    #[serde(alias = "g", alias = "G", alias = "gb", alias = "GB")]
    GB,
}

#[derive(Deserialize, Debug)]
pub enum Log {
    // level: log::Level,
// sink: uri
}
#[derive(Deserialize, Debug, Copy, Clone)]
pub struct Baud(pub u32);

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct Frequency(pub u32);

impl Frequency {
    pub fn from((value, unit): &(u32, UnitHz)) -> Frequency {
        match unit {
            UnitHz::Hz => Self::hertz(*value),
            UnitHz::KHz => Self::kilo_hertz(*value),
            UnitHz::MHz => Self::mega_hertz(*value),
            UnitHz::GHz => Self::giga_hertz(*value),
        }
    }
    pub fn hertz(hertz: u32) -> Frequency {
        Frequency(hertz)
    }
    pub fn kilo_hertz(hertz: u32) -> Frequency {
        Frequency(hertz * 1000)
    }
    pub fn mega_hertz(hertz: u32) -> Frequency {
        Frequency(hertz * 1000 * 1000)
    }
    pub fn giga_hertz(hertz: u32) -> Frequency {
        Frequency(hertz * 1000 * 1000 * 1000)
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Deserialize)]
pub enum Direction {
    #[serde(alias = "input", alias = "INPUT")]
    Input,
    #[serde(alias = "output", alias = "OUTPUT")]
    Output,
    Alternate,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Deserialize)]
pub enum TriggerEdge {
    #[serde(
        alias = "Interrupt",
        alias = "INTERRUPT",
        alias = "interrupt",
        alias = "Rising",
        alias = "RISING",
        alias = "rising"
    )]
    Rising,
    #[serde(alias = "FALLING", alias = "falling")]
    Falling,
    #[serde(
        alias = "ALL",
        alias = "all",
        alias = "RisingFalling",
        alias = "RISINGFALLING",
        alias = "risingfalling",
        alias = "rising_falling"
    )]
    All,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Deserialize)]
pub enum PinMode {
    #[serde(alias = "analog", alias = "ANALOG")]
    Analog,
    #[serde(alias = "floating", alias = "FLOATING")]
    Floating,
    #[serde(alias = "open_drain", alias = "opendrain", alias = "OPENDRAIN")]
    OpenDrain,
    #[serde(alias = "pull_down", alias = "pulldown", alias = "PULLDOWN")]
    PullDown,
    #[serde(alias = "pull_up", alias = "pullup", alias = "PULLUP")]
    PullUp,
    #[serde(alias = "push_pull", alias = "pushpull", alias = "PUSHPULL")]
    PushPull,
}

impl Direction {
    pub fn to_type_string(&self) -> String {
        match self {
            Direction::Input => "Input",
            Direction::Output => "Output",
            Direction::Alternate => "Alternate",
        }
        .into()
    }
}
impl PinMode {
    pub fn to_string(&self) -> String {
        match self {
            PinMode::Analog => "analog",
            PinMode::Floating => "floating",
            PinMode::OpenDrain => "open_drain",
            PinMode::PullDown => "pull_down",
            PinMode::PullUp => "pull_up",
            PinMode::PushPull => "push_pull",
        }
        .into()
    }
    pub fn to_type_string(&self) -> String {
        match self {
            PinMode::Analog => "Analog",
            PinMode::Floating => "Floating",
            PinMode::OpenDrain => "OpenDrain",
            PinMode::PullDown => "PullDown",
            PinMode::PullUp => "PullUp",
            PinMode::PushPull => "PushPull",
        }
        .into()
    }
}
