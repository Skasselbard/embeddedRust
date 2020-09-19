use super::{Pin, Timer, Generator};
use crate::types::{UnitHz, PWMInterface, Frequency};
use serde_derive::Deserialize;
use crate::generation::PWMGeneration;
use syn::{Stmt, parse_quote, parse_str};
use quote::format_ident;

/// ```
/// "pwm":[{
///     "timer":    "Tim2", 
///     "pins":     ["PA1"], 
///     "frequency":[10,"khz"]
/// }]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct PWM {
    timer: Timer,
    pins: Vec<Pin>,
    frequency: (u32, UnitHz),
}



impl PWMInterface for PWM {
    fn pins(&self) -> Vec<&dyn crate::types::Pin> {
        self.pins.iter().map(|pin|pin as &dyn crate::types::Pin).collect()
    }

    fn tys(&self) -> Vec<syn::Type> {
        self.pins.iter().map(|pin|{
            let timer = match self.timer{
                Timer::Tim1 => {"TIM1"},
                Timer::Tim2 => {"TIM2"},
                Timer::Tim3 => {"TIM3"},
                Timer::Tim4 => {"TIM4"},
            };
            parse_str(&format!(
                "PwmChannel<pac::{}, pwm::{}>",
                timer,
                self.timer.channel(pin)
            )).unwrap()
        }).collect()
    }

    fn frequency(&self) -> crate::types::Frequency {
        Frequency::from(&self.frequency)
    }

    fn generate(&self) -> Vec<Stmt> {
        let mut stmts = vec![];
        let peripherals = peripherals_ident!();
        let timer = format_ident!("{}", self.timer.name());
        let timer_upper = format_ident!("{}", self.timer.name().to_uppercase());
        let apb = format_ident!("{}", self.timer.peripheral_bus());
        let frequency = Frequency::from(&self.frequency).0;
        let mut pin_ids = vec![];
        for pin in &self.pins{
            use crate::types::Pin;
            let channel = format_ident!("{}", pin.channel_name());
            let pin_name= format_ident!("{}", pin.name());
            let ctrl_reg = format_ident!("{}", super::gpio::control_reg(pin as &dyn crate::types::Pin));
            // expands to:
            // ``let pxy = gpiox.pxy.into_alternate_push_pull(&mut gpiox.crl);``
            stmts.append(&mut parse_quote!(
                let #pin_name = #channel.#pin_name.into_alternate_push_pull(&mut #channel.#ctrl_reg);
            ));
            pin_ids.push(pin_name);
        }
        // expands to:
        // ```
        // let pwx = pwx.into_alternate_push_pull(&mut gpiow.crl);
        // let pyz = gpioy.pyz.into_alternate_push_pull(&mut gpioy.crl);
        // let timer = Timer::timx(p.TIMX, &clocks, &mut rcc.apb);
        // let (pwx, pyz) = timer.pwm((pwx, pyz), &mut afio.mapr, freq);
        // ```
        stmts.append(&mut parse_quote!(
            let timer = Timer::#timer(#peripherals.#timer_upper, &clocks, &mut rcc.#apb);
            let (#(#pin_ids),*) = timer.pwm((#(#pin_ids),*), &mut afio.mapr, #frequency.hz()).split();
        ));
        stmts
    }
}

impl PWMGeneration for Generator{}
