#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         //extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
                         //extern crate stm32f1xx_hal;

use cortex_m_rt::entry;
//use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::serial::Serial;

use nb::block;

use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let mut flash = p.FLASH.constrain();
    //let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let uart_pins = (
        gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl),
        gpiob.pb7,
    );

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut timer = Timer::tim1(p.TIM1, 1.hz(), clocks, &mut rcc.apb2);

    let (mut send, mut rec) = Serial::usart1(
        p.USART1,
        uart_pins,
        &mut afio.mapr,
        115200_u32.bps(),
        clocks,
        &mut rcc.apb2,
    )
    .split();
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
        block!(send.write(b'a')).ok();
        let received = block!(rec.read()).unwrap();
        assert_eq!(received, b'c')
    }
}
