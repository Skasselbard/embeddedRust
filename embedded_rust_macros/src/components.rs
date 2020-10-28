use crate::Config;
use syn::{parse_str, Expr, Ident, Stmt, Type};

pub trait Component {
    fn identifier(&self) -> Ident;
    fn ty(&self) -> Type;
}

pub(crate) struct Components {
    pub init_stmts: Vec<Stmt>,
    pub interrupt_unmasks: Vec<Stmt>,

    pub sys: Sys,
    pub input_pins: InPins,
    pub output_pins: OutPins,
    pub pwm_pins: PWMPins,
    pub channels: Channels,
    pub serials: Serials,
    pub timers: Timers,
}

pub(crate) struct Sys {
    pub identifiers: Vec<Ident>,
    pub ty: Vec<Type>,
}
pub(crate) struct InPins {
    pub identifiers: Vec<Ident>,
    pub channels: Vec<Expr>,
    pub ports: Vec<Expr>,
    pub ty: Vec<Type>,
}
pub(crate) struct OutPins {
    pub identifiers: Vec<Ident>,
    pub channels: Vec<Expr>,
    pub ports: Vec<Expr>,
    pub ty: Vec<Type>,
}
pub(crate) struct PWMPins {
    pub identifiers: Vec<Ident>,
    pub channels: Vec<Expr>,
    pub ports: Vec<Expr>,
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
    Components {
        init_stmts: config.init_statements(),
        interrupt_unmasks: config.interrupt_unmasks(),
        sys: Sys {
            identifiers: vec![],
            ty: vec![],
        },
        input_pins: InPins {
            identifiers: config.input_idents(),
            ty: config.input_tys(),
            channels: config.input_channels(),
            ports: config.input_ports(),
        },
        output_pins: OutPins {
            identifiers: config.output_idents(),
            ty: config.output_tys(),
            channels: config.output_channels(),
            ports: config.output_ports(),
        },
        pwm_pins: PWMPins {
            identifiers: config.pwm_idents(),
            channels: config.pwm_channels(),
            ports: config.pwm_ports(),
            ty: config.pwm_tys(),
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
