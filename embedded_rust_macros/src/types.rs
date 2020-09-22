use quote::format_ident;
use serde_derive::Deserialize;

use crate::devices::{dummy, stm32f1xx};
use crate::generation::Generator;
use crate::types;
use crate::Component;
use syn::{Expr, Ident, Stmt, Type};

/// This is the struct that is parsed from the macro input.
/// It is an enum where each variant determines the different boards.
/// The content of the variants should be identical for each board.
/// A simple example is the dummy variant. An example device implementation is
/// located in the devices::dummy module. If you would like to implement an
/// additional device, you can copy and iteratively expand the dummy implementation to
/// have a start point.
#[derive(Deserialize, Debug)]
pub enum Config {
    #[serde(alias = "dummy", alias = "DUMMY")]
    Dummy {
        #[serde(alias = "sys")]
        sys: types::Sys,
        #[serde(default, alias = "gpios")]
        gpios: Vec<dummy::DummyGpio>,
        pwm: Vec<dummy::DummyPWM>,
    },
    #[serde(
        alias = "stm32f1",
        alias = "STM32F1",
        alias = "Stm32F1",
        alias = "STM32f1",
        alias = "BLUEPILL",
        alias = "blue_pill",
        alias = "BluePill",
        alias = "bluepill"
    )]
    Stm32f1xx {
        #[serde(alias = "sys")]
        sys: types::Sys,
        #[serde(default, alias = "gpios")]
        gpios: Vec<stm32f1xx::StmGpio>,
        pwm: Vec<stm32f1xx::PWM>,
    },
}

impl Config {
    pub fn sys(&self) -> &types::Sys {
        match self {
            Config::Dummy { sys, .. } => sys,
            Config::Stm32f1xx { sys, .. } => sys,
        }
    }
    pub fn gpios(&self) -> Vec<&dyn Gpio> {
        match self {
            Config::Dummy { gpios, .. } => gpios.iter().map(|gpio| gpio as &dyn Gpio).collect(),
            Config::Stm32f1xx { gpios, .. } => gpios.iter().map(|gpio| gpio as &dyn Gpio).collect(),
        }
    }
    fn pwm(&self) -> Vec<&dyn PWMInterface> {
        match self {
            Config::Dummy { pwm, .. } => pwm.iter().map(|pwm| pwm as &dyn PWMInterface).collect(),
            Config::Stm32f1xx { pwm, .. } => {
                pwm.iter().map(|pwm| pwm as &dyn PWMInterface).collect()
            }
        }
    }
    pub fn generator(&self) -> &dyn Generator {
        match self {
            Config::Dummy { .. } => &dummy::DummyGenerator,
            Config::Stm32f1xx { .. } => &stm32f1xx::Generator,
        }
    }
    pub fn init_statements(&self) -> Vec<Stmt> {
        let code_gen = self.generator();
        let mut init_stmts = code_gen.generate_imports();
        init_stmts.append(&mut code_gen.generate_device_init());
        init_stmts.append(
            &mut code_gen
                .generate_clock(&self.sys().sys_clock.as_ref().map(|f| Frequency::from(f))),
        );
        init_stmts.append(&mut code_gen.generate_channels(&self.gpios()));
        init_stmts.append(&mut code_gen.generate_gpios(&self.gpios()));
        init_stmts.append(&mut code_gen.generate_pwm_pins(&self.pwm()));
        init_stmts
    }
    pub fn interrupt_unmasks(&self) -> Vec<Stmt> {
        self.generator().interrupts(&self.gpios())
    }
    fn output_pins(&self) -> Vec<&dyn Gpio> {
        let (out_pins, _in_pins): (Vec<&dyn Gpio>, Vec<&dyn Gpio>) = self
            .gpios()
            .iter()
            .partition(|gpio| gpio.direction() == &Direction::Output);
        out_pins
    }
    fn input_pins(&self) -> Vec<&dyn Gpio> {
        let (_out_pins, in_pins): (Vec<&dyn Gpio>, Vec<&dyn Gpio>) = self
            .gpios()
            .iter()
            .partition(|gpio| gpio.direction() == &Direction::Output);
        in_pins
    }
    pub fn input_channels(&self) -> Vec<Expr> {
        self.input_pins()
            .iter()
            .map(|gpio| gpio.pin().channel_constructor())
            .collect()
    }
    pub fn input_ports(&self) -> Vec<Expr> {
        self.input_pins()
            .iter()
            .map(|gpio| gpio.pin().port_constructor())
            .collect()
    }

    pub fn output_channels(&self) -> Vec<Expr> {
        self.output_pins()
            .iter()
            .map(|gpio| gpio.pin().channel_constructor())
            .collect()
    }
    pub fn output_ports(&self) -> Vec<Expr> {
        self.output_pins()
            .iter()
            .map(|gpio| gpio.pin().port_constructor())
            .collect()
    }
    pub fn input_idents(&self) -> Vec<Ident> {
        self.input_pins()
            .iter()
            .map(|gpio| gpio.identifier())
            .collect()
    }
    pub fn input_tys(&self) -> Vec<Type> {
        self.input_pins().iter().map(|gpio| gpio.ty()).collect()
    }
    pub fn output_idents(&self) -> Vec<Ident> {
        self.output_pins()
            .iter()
            .map(|gpio| gpio.identifier())
            .collect()
    }
    pub fn output_tys(&self) -> Vec<Type> {
        self.output_pins().iter().map(|gpio| gpio.ty()).collect()
    }
    fn pwm_pins(&self) -> Vec<&dyn Pin> {
        self.pwm().iter().map(|pwm| pwm.pins()).flatten().collect()
    }
    pub fn pwm_idents(&self) -> Vec<Ident> {
        self.pwm_pins()
            .iter()
            .map(|pin| format_ident!("{}", pin.name()))
            .collect()
    }
    pub fn pwm_channels(&self) -> Vec<Expr> {
        self.pwm_pins()
            .iter()
            .map(|pin| pin.channel_constructor())
            .collect()
    }
    pub fn pwm_ports(&self) -> Vec<Expr> {
        self.pwm_pins()
            .iter()
            .map(|pin| pin.port_constructor())
            .collect()
    }
    pub fn pwm_tys(&self) -> Vec<Type> {
        self.pwm().iter().map(|pwm| pwm.tys()).flatten().collect()
    }
}

/// Gpios are device dependend, so they have some general behavior that they should provide.
/// Currently it is expected that each gpio has a pin (covert with
/// an additional trait), a direction (in/out), an interrupt trigger edge configuration
/// and a mode. It is useful to structure the actual gpio type in a similar way (see
/// dummy example).
pub trait Gpio: Component {
    fn pin(&self) -> &dyn Pin;
    fn direction(&self) -> &Direction;
    fn mode(&self) -> &PinMode;
    fn trigger_edge(&self) -> Option<TriggerEdge>;
}

pub trait PWMInterface {
    fn pins(&self) -> Vec<&dyn Pin>;
    fn tys(&self) -> Vec<Type>;
    fn frequency(&self) -> Frequency;
    fn generate(&self) -> Vec<syn::Stmt>;
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
