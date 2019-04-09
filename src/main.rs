#![no_std]
#![no_main]

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
                                //extern crate stm32f1xx_hal;

//use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
//use stm32f1xx_hal as hal;
//use stm32f1xx_hal::serial::{Serial};
//use stm32f1::stm32f103 as stm32;

use nb::block;

use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    //let mut rcc = dp.RCC.constrain();

    // let clocks = rcc
    //     .cfgr
    //     .use_hse(32.khz())
    //     .sysclk(72.mhz())
    //     .freeze(&mut flash.acr);

    // // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // // `clocks`
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // // Acquire the GPIOC peripheral
    // let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // // in order to configure the port. For pins 0-7, crl should be passed instead.
    // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // // Configure the syst timer to trigger an update every second
    // let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);

    // // Wait for the timer to trigger an update and change the state of the LED
    loop {
        hprintln!("Hello, world!").unwrap();
        // block!(timer.wait()).unwrap();
        // led.set_high();
        // block!(timer.wait()).unwrap();
        // led.set_low();
    }
}
// #[entry]
// fn main() -> ! {
//     let p = hal::stm32::Peripherals::take().unwrap();
//     let cp = hal::stm32::CorePeripherals::take().unwrap();
//     let mut rcc = p.RCC.constrain();
//     let mut flash = p.FLASH.constrain();
//     //let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
//     let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
//     let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
//     let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
//     let mut afio = p.AFIO.constrain(&mut rcc.apb2);
//     let mut uartPins = (
//         gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl),
//         gpiob.pb7.into_floating_input(&mut gpiob.crl)
//         );

//     let clocks = rcc
//         .cfgr
//         .use_hse(8.mhz())
//         .sysclk(16.mhz())
//         .freeze(&mut flash.acr);

//     let (mut send, rec) = Serial::usart1(
//         p.USART1,
//         uartPins,
//         &mut afio.mapr,
//         115200_u32.bps(),
//         clocks,
//         &mut rcc.apb2
//     ).split();
//     loop {
//         send.write(b'a');
//         send.flush();
//     }
// }
