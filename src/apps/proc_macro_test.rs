#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_rust::*;
use embedded_rust_macros::*;

pub const HEAP_START: usize = 0x2000_0000;
pub const HEAP_SIZE: usize = 10 * 1024; // 10 KiB

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("panic: {}", info);
    // interrupt::disable();
    loop {}
}

#[entry]
fn main() -> ! {
    let (configurations, init_closure) = configure_device!({
            "gpios": [
                ["PA0", "input", "floating", "interrupt"],
                ["PB10", "output", "push_pull"]
            ]
    });
    let runtime = Runtime::init(HEAP_START, HEAP_SIZE, 32, &configurations, init_closure);
    runtime.expect("InitError").run();
}
