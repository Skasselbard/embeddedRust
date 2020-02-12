//! Turns the user LED on
//!
//! Listens for interrupts on the pa7 pin. On any rising or falling edge, toggles
//! the pc13 pin (which is connected to the LED on the blue pill board, hence the `led` name).

#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::{
    adc, pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

mod tasks;

#[entry]
fn main() -> ! {
    // initialization phase
    let p = pac::Peripherals::take().unwrap();
    // let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    // let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    // pwm
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let pwm = Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
        c1,
        &mut afio.mapr,
        1.khz(),
    );

    // adc
    let adc = adc::Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);
    let adc_max_sample = adc.max_sample();
    let channel = gpioa.pa7.into_analog(&mut gpioa.crl);

    let mut prog = tasks::Prog::init(pwm, adc, adc_max_sample, channel);
    prog.run()
}
