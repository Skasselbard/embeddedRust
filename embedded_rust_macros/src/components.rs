use crate::types::Gpio;
use crate::Config;
use crate::{Direction, Frequency};
use syn::{parse_str, Expr, Ident, Stmt, Type};

pub trait Component {
    fn identifier(&self) -> Ident;
    fn ty(&self) -> Type;
}

pub(crate) struct Components<'c> {
    pub init_stmts: Vec<Stmt>,
    pub interrupt_unmasks: Vec<Stmt>,

    pub sys: Sys,
    pub input_pins: InPins<'c>,
    pub output_pins: OutPins<'c>,
    pub pwms: PWMs,
    pub channels: Channels,
    pub serials: Serials,
    pub timers: Timers,
}

pub(crate) struct Sys {
    pub identifiers: Vec<Ident>,
    pub parsed_data: (),
    pub ty: Vec<Type>,
}
pub(crate) struct InPins<'c> {
    pub identifiers: Vec<Ident>,
    pub parsed_data: Vec<&'c dyn Gpio>,
    pub channels: Vec<Expr>,
    pub ports: Vec<Expr>,
    pub ty: Vec<Type>,
    pub error_ty: Type,
}
pub(crate) struct OutPins<'c> {
    pub identifiers: Vec<Ident>,
    pub parsed_data: Vec<&'c dyn Gpio>,
    pub channels: Vec<Expr>,
    pub ports: Vec<Expr>,
    pub ty: Vec<Type>,
    pub error_ty: Type,
}
pub(crate) struct PWMs {
    pub identifiers: Vec<Ident>,
    pub parsed_data: (),
    pub ty: Vec<Type>,
}
pub(crate) struct Channels {
    pub identifiers: Vec<Ident>,
    pub parsed_data: (),
    pub ty: Vec<Type>,
    pub id_ty: Type,
}
pub(crate) struct Serials {
    pub identifiers: Vec<Ident>,
    pub parsed_data: (),
    pub ty: Vec<Type>,
}
pub(crate) struct Timers {
    pub identifiers: Vec<Ident>,
    pub parsed_data: (),
    pub ty: Vec<Type>,
}

pub(crate) fn parse_components(config: &Config) -> Components {
    let gpios = config.gpios();
    let (out_pins, in_pins): (Vec<&dyn Gpio>, Vec<&dyn Gpio>) = gpios
        .iter()
        .partition(|gpio| gpio.direction() == &Direction::Output);
    let sys_freq = config.sys().sys_clock.as_ref().map(|f| Frequency::from(f));
    let code_gen = config.generator();
    let mut init_stmts = code_gen.generate_imports();
    init_stmts.append(&mut code_gen.generate_device_init());
    init_stmts.append(&mut code_gen.generate_clock(&sys_freq));
    init_stmts.append(&mut code_gen.generate_channels(&gpios));
    init_stmts.append(&mut code_gen.generate_gpios(&gpios));
    let interrupt_unmasks = code_gen.interrupts(&gpios);
    // generate Channel and port constructors
    let (in_channels, in_ports) = in_pins
        .iter()
        .map(|gpio| {
            (
                gpio.pin().channel_constructor(),
                gpio.pin().port_constructor(),
            )
        })
        .unzip();
    let (out_channels, out_ports) = out_pins
        .iter()
        .map(|gpio| {
            (
                gpio.pin().channel_constructor(),
                gpio.pin().port_constructor(),
            )
        })
        .unzip();
    Components {
        init_stmts,
        interrupt_unmasks,
        sys: Sys {
            identifiers: vec![],
            parsed_data: (),
            ty: vec![],
        },
        input_pins: InPins {
            identifiers: in_pins.iter().map(|gpio| gpio.identifier()).collect(),
            ty: in_pins.iter().map(|gpio| gpio.ty()).collect(),
            channels: in_channels,
            ports: in_ports,
            parsed_data: in_pins,
            error_ty: code_gen.input_error(),
        },
        output_pins: OutPins {
            identifiers: out_pins.iter().map(|gpio| gpio.identifier()).collect(),
            ty: out_pins.iter().map(|gpio| gpio.ty()).collect(),
            channels: out_channels,
            ports: out_ports,
            parsed_data: out_pins,
            error_ty: code_gen.output_error(),
        },
        pwms: PWMs {
            identifiers: vec![],
            parsed_data: (),
            ty: vec![],
        },
        channels: Channels {
            identifiers: vec![],
            parsed_data: (),
            ty: vec![],
            id_ty: parse_str("()").unwrap(),
        },
        serials: Serials {
            identifiers: vec![],
            parsed_data: (),
            ty: vec![],
        },
        timers: Timers {
            identifiers: vec![],
            parsed_data: (),
            ty: vec![],
        },
    }
}
