#[macro_use]
mod usart;
mod gpio;
mod pwm;
pub use gpio::*;
use nom_uri::ToUri;
pub use pwm::*;
pub use usart::*;

pub type DeviceInterrupt = stm32f1xx_hal::device::Interrupt;

/// The heap starts after the data segments of static values (.data and .bss)
/// #[link_section] places the annotated static directly at the given data segment.
/// We can use the adress of this static to determine the start of the heap
/// if we use the .uninit segment (unoccupied data after .bss) as section.
/// See the [cortex-m-rt documentation](https://docs.rs/cortex-m-rt/0.6.12/cortex_m_rt/#uninitialized-static-variables) and [link section reference](https://doc.rust-lang.org/reference/abi.html#the-link_section-attribute) for mor information
pub fn heap_bottom() -> usize {
    #[link_section = ".uninit"]
    static HEAP_BOTTOM: usize = 0;
    &HEAP_BOTTOM as *const usize as usize
}

pub fn sleep() {
    cortex_m::asm::wfe()
}

pub enum ComponentConfiguration {
    Clock,
    Gpio(Gpio),
    Usart,
    Pwm,
}

impl<'uri> ToUri<'uri> for ComponentConfiguration {
    fn to_uri(&self, buffer: &'uri mut str) -> nom_uri::Uri<'uri> {
        match self {
            ComponentConfiguration::Gpio(gpio) => gpio.to_uri(buffer),
            _ => unimplemented!(),
        }
    }
}

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
