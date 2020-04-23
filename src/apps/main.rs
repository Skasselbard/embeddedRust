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
    adc,
    device::interrupt,
    pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

use embedded_rust::device::{self};
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
    // initialization phase
    let p = pac::Peripherals::take().unwrap();
    // let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut runtime = Runtime::init(HEAP_START, HEAP_SIZE, 32).unwrap();
    let mut usart1 = usart1!(gpioa, p, rcc, afio, clocks);
    #[interrupt]
    fn TIM2() {
        // unsafe { x += 1 };
    }
    let mut pwm = pwm_tim3!(gpioa, gpiob, p, rcc, afio, clocks, ;gpiob.pb0);
    let (mut adc, mut channels) = adc1!(gpioa, p, rcc, clocks, gpioa.pa7);
    let id = runtime.add_resource(nom_uri::Uri::parse("bus:uart/1").unwrap(), &mut usart1);
    runtime.associate_interrupt(id, DeviceInterrupt::USART1);
    let mut task = example_task();
    runtime.spawn_task(Task::new(task), 0);
    runtime.run()
}
async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    hprintln!("async number: {}", number);
}
