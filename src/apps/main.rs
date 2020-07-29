#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

use core::panic::PanicInfo;
//use cortex_m::interrupt;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::{
    adc, pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

use embedded_rust::*;

pub const HEAP_START: usize = 0x2000_0000;
pub const HEAP_SIZE: usize = 10 * 1024; // 10 KiB

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("panic: {}", info);
    // interrupt::disable();
    loop {}
}

// static mut x: usize = 0;
#[entry]
fn main() -> ! {
    // let mut runtime = Runtime::init(HEAP_START, HEAP_SIZE, 32).unwrap();
    // // let mut usart1 = usart1!(gpioa, p, rcc, afio, clocks);
    // //let mut pwm = pwm_tim2!(gpioa, p, rcc, afio, clocks, gpioa.pa0);
    // // let (mut adc, mut channels) = adc1!(gpioa, p, rcc, clocks, gpioa.pa7);
    // // let id = runtime.add_resource(nom_uri::Uri::parse("bus:uart/1").unwrap(), &mut usart1);
    // // runtime.associate_interrupt(id, DeviceInterrupt::USART1);

    // // let mut gpio = build_gpio!(device, pa0, input, floating);
    // // gpio.enable_interrupt(&mut afio);

    // let mut task = example_task();
    // runtime.spawn_task(Task::new(task), 0);
    // runtime.run()
    loop {}
}

async fn switch_pwm() {}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    hprintln!("async number: {}", number);
}
