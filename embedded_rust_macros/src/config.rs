use quote::format_ident;
use serde_derive::Deserialize;

use crate::devices::{dummy, stm32f1xx};
use crate::generation::Generator;
use crate::types;
use syn::{parse_quote, Expr, Ident, Stmt, Type};
use types::{Direction, Frequency, Gpio, PWMInterface, Pin, Serial};

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
        #[serde(default)]
        pwm: Vec<dummy::DummyPWM>,
        #[serde(default)]
        serial: Vec<dummy::DummySerial>,
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
        #[serde(default)]
        pwm: Vec<stm32f1xx::PWM>,
        #[serde(default)]
        serial: Vec<stm32f1xx::StmSerial>,
    },
}

impl Config {
    pub fn sys(&self) -> &types::Sys {
        match self {
            Config::Dummy { sys, .. } => sys,
            Config::Stm32f1xx { sys, .. } => sys,
        }
    }
    pub fn gpios(&self) -> Vec<Box<dyn Gpio>> {
        match self {
            Config::Dummy { gpios, .. } => gpios
                .iter()
                .map(|gpio| Box::new(*gpio) as Box<dyn Gpio>)
                .collect(),
            Config::Stm32f1xx { gpios, .. } => gpios
                .iter()
                .map(|gpio| Box::new(*gpio) as Box<dyn Gpio>)
                .collect(),
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
    fn serials(&self) -> Vec<&dyn Serial> {
        match self {
            Config::Dummy { serial, .. } => {
                serial.iter().map(|serial| serial as &dyn Serial).collect()
            }
            Config::Stm32f1xx { serial, .. } => {
                serial.iter().map(|serial| serial as &dyn Serial).collect()
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
        let mut all_gpios: Vec<Box<dyn Gpio>> = self.gpios();
        all_gpios.append(&mut self.pwm_gpios());
        all_gpios.append(&mut self.serial_gpios());

        let code_gen = self.generator();
        let mut init_stmts = code_gen.generate_imports();
        init_stmts.append(&mut code_gen.generate_device_init());
        init_stmts.append(
            &mut code_gen
                .generate_clock(&self.sys().sys_clock.as_ref().map(|f| Frequency::from(f))),
        );
        init_stmts.append(&mut code_gen.generate_channels(&all_gpios));
        init_stmts.append(&mut code_gen.generate_gpios(&all_gpios));
        init_stmts.append(&mut code_gen.generate_pwm_pins(&self.pwm()));
        init_stmts.append(&mut code_gen.generate_serials(&self.serials()));
        init_stmts
    }
    pub fn interrupt_unmasks(&self) -> Vec<Stmt> {
        self.generator().interrupts(&self.gpios())
    }
    fn output_pins(&self) -> Vec<Box<dyn Gpio>> {
        let (out_pins, _in_pins): (Vec<Box<dyn Gpio>>, Vec<Box<dyn Gpio>>) = self
            .gpios()
            .into_iter()
            .partition(|gpio| gpio.direction() == &Direction::Output);
        out_pins
    }
    fn input_pins(&self) -> Vec<Box<dyn Gpio>> {
        let (_out_pins, in_pins): (Vec<Box<dyn Gpio>>, Vec<Box<dyn Gpio>>) = self
            .gpios()
            .into_iter()
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
    pub fn input_constructors(&self) -> Vec<Expr> {
        let channels = self.input_channels().into_iter();
        let ports = self.input_ports().into_iter();
        let idents = self.input_idents().into_iter();
        let mut constructors = vec![];
        for (channel, (port, ident)) in channels.zip(ports.zip(idents)) {
            constructors.push(parse_quote!(InputPin::new(Pin::new(#channel, #port), #ident)));
        }
        constructors
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
    pub fn output_constructors(&self) -> Vec<Expr> {
        let channels = self.output_channels().into_iter();
        let ports = self.output_ports().into_iter();
        let idents = self.output_idents().into_iter();
        let mut constructors = vec![];
        for (channel, (port, ident)) in channels.zip(ports.zip(idents)) {
            constructors.push(parse_quote!(OutputPin::new(Pin::new(#channel, #port), #ident)));
        }
        constructors
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
    pub fn pwm_constructors(&self) -> Vec<Expr> {
        let channels = self.pwm_channels().into_iter();
        let ports = self.pwm_ports().into_iter();
        let idents = self.pwm_idents().into_iter();
        let mut constructors = vec![];
        for (channel, (port, ident)) in channels.zip(ports.zip(idents)) {
            constructors.push(parse_quote!(
                   PWMPin::new(Pin::new(#channel , #port), #ident)
            ))
        }
        constructors
    }
    fn pwm_gpios(&self) -> Vec<Box<dyn Gpio>> {
        self.pwm()
            .into_iter()
            .map(|pwm| pwm.pins_as_gpios())
            .fold(vec![], |mut res, mut next| {
                res.append(&mut next);
                res
            })
    }

    pub fn serial_rx_pins(&self) -> Vec<&dyn Pin> {
        self.serials()
            .iter()
            .map(|serial| serial.receive_pin())
            .collect()
    }
    pub fn serial_tx_pins(&self) -> Vec<&dyn Pin> {
        self.serials()
            .iter()
            .map(|serial| serial.transmit_pin())
            .collect()
    }
    pub fn serial_idents(&self) -> Vec<Ident> {
        self.serials()
            .iter()
            .map(|serial| format_ident!("{}", serial.name()))
            .collect()
    }
    fn serial_gpios(&self) -> Vec<Box<dyn Gpio>> {
        self.serials()
            .into_iter()
            .flat_map(|serial| serial.pins_as_gpio())
            .collect()
    }
    pub fn serial_tys(&self) -> Vec<Type> {
        self.serials().iter().map(|serial| serial.ty()).collect()
    }
    pub fn serial_word_tys(&self) -> Vec<Type> {
        self.serials()
            .iter()
            .map(|serial| serial.word_ty())
            .collect()
    }
    pub fn serial_constructors(&self) -> Vec<Expr> {
        let mut constructors = vec![];
        for serial in self.serials() {
            let ident = format_ident!("{}", serial.name());
            let serial_id = format_ident!("{}", serial.serial_id());
            constructors.push(parse_quote!(
                   Serial::new(SerialID::#serial_id, #ident)
            ))
        }
        constructors
    }
}
