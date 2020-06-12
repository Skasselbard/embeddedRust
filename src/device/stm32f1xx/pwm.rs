/// ## Example
/// [Reference](https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/pwm.rs)
/// ```
/// pwm_tim2!(gpioa, gpiob, p, rcc, afio, clocks, gpioa.pa0);
/// pwm_tim2!(gpioa, gpiob, p, rcc, afio, clocks, gpioa.pb0, gpioa.pa3);
/// pwm_tim2!(gpioa, gpiob, p, rcc, afio, clocks, gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3);
/// ```
#[macro_export]
macro_rules! pwm_tim2 {
    ( $gpioa:expr, $peripherals:expr, $rcc:expr, $afio:expr, $clocks:expr, $($pin:expr),+) => {{
        let pins = ($($pin.into_alternate_push_pull(&mut $gpioa.crl)),+);
        Timer::tim2($peripherals.TIM2, &$clocks, &mut $rcc.apb1)
            .pwm::<stm32f1xx_hal::timer::Tim2NoRemap, _, _, _>(pins, &mut $afio.mapr, 1.khz())
    }};
}

/// ## Example
/// [Reference](https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/pwm.rs)
/// ```
/// pwm_tim3!(gpioa, gpiob, p, rcc, afio, clocks, gpioa.pa6;);
/// pwm_tim3!(gpioa, gpiob, p, rcc, afio, clocks, ;gpiob.pb0);
/// pwm_tim3!(gpioa, gpiob, p, rcc, afio, clocks, gpioa.pa6, gpioa.pa7 ;gpiob.pb0, gpiob.pb1);
/// ```
#[macro_export]
macro_rules! pwm_tim3 {
    ( $gpioa:expr, $gpiob:expr, $peripherals:expr, $rcc:expr, $afio:expr, $clocks:expr, $($pina:expr),* ; $($pinb:expr),*) => {{
        let pins = ($($pina.into_alternate_push_pull(&mut $gpioa.crl)),* $($pinb.into_alternate_push_pull(&mut $gpiob.crl)),*);
        Timer::tim3($peripherals.TIM3, &$clocks, &mut $rcc.apb1)
            .pwm::<stm32f1xx_hal::timer::Tim3NoRemap, _, _, _>(pins, &mut $afio.mapr, 1.khz())
    }};
}
