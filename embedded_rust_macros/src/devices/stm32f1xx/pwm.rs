use super::{Generator, Pin, StmGpio, Timer};
use crate::types::{Frequency, Gpio, PWMInterface, UnitHz};
use crate::{
    generation::PWMGeneration,
    types::{Direction, PinMode},
};
use quote::format_ident;
use serde_derive::Deserialize;
use syn::{parse_quote, parse_str, Ident, Stmt};

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
        self.pins
            .iter()
            .map(|pin| pin as &dyn crate::types::Pin)
            .collect()
    }

    fn tys(&self) -> Vec<syn::Type> {
        self.pins
            .iter()
            .map(|pin| {
                let timer = match self.timer {
                    Timer::Tim1 => "TIM1",
                    Timer::Tim2 => "TIM2",
                    Timer::Tim3 => "TIM3",
                    Timer::Tim4 => "TIM4",
                };
                parse_str(&format!(
                    "PwmChannel<pac::{}, pwm::{}>",
                    timer,
                    self.timer.channel(pin)
                ))
                .unwrap()
            })
            .collect()
    }

    fn frequency(&self) -> crate::types::Frequency {
        Frequency::from(&self.frequency)
    }

    // expands to:
    // ```
    // let timer = Timer::timx(p.TIMX, &clocks, &mut rcc.apb);
    // let (pwx, pyz) = timer.pwm((pwx, pyz), &mut afio.mapr, freq);
    // ```
    fn generate(&self) -> Vec<Stmt> {
        let peripherals = peripherals_ident!();
        let timer = format_ident!("{}", self.timer.name());
        let timer_upper = format_ident!("{}", self.timer.name().to_uppercase());
        let timer_remap = format_ident!("{}", self.timer.remap(&self.pins));
        let apb = format_ident!("{}", self.timer.peripheral_bus());
        let frequency = Frequency::from(&self.frequency).0;
        let pin_ids: Vec<Ident> = self
            .pins()
            .iter()
            .map(|pin| format_ident!("{}", pin.name()))
            .collect();
        parse_quote!(
            let timer = Timer::#timer(#peripherals.#timer_upper, &clocks, &mut rcc.#apb);
            let (#(#pin_ids),*) = timer.pwm::<timer::#timer_remap, _, _, _>((#(#pin_ids),*), &mut afio.mapr, #frequency.hz()).split();
        )
    }

    fn pins_as_gpios(&self) -> Vec<Box<dyn crate::types::Gpio>> {
        self.pins
            .iter()
            .map(|pin| {
                Box::new(StmGpio::new(
                    *pin,
                    Direction::Alternate,
                    PinMode::PushPull,
                    None,
                )) as Box<dyn Gpio>
            })
            .collect()
    }
}

impl PWMGeneration for Generator {}
