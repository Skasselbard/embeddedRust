#[macro_use]
mod usart;
mod pwm;
pub use pwm::*;
pub use usart::*;

/// consider configuring clocks before adc construction:
/// ```
/// let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);
/// ```
/// ## return
/// (adc, (channel1, channel2, .. ))
#[macro_export]
macro_rules! adc1 {
    ($gpioa:expr, $peripherals:expr, $rcc:expr, $clocks:expr, $($pina:expr),+) => {{
        let adc = adc::Adc::adc1($peripherals.ADC1, &mut $rcc.apb2, $clocks);
        (adc, ($($pina.into_analog(&mut $gpioa.crl)),+))
    }};
}
