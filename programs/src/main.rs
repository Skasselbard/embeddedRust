#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_rust::*;
use embedded_rust_macros::*;
use futures::StreamExt;

pub const HEAP_START: usize = 0x2000_0000;
pub const HEAP_SIZE: usize = 10 * 1024; // 10 KiB

#[entry]
fn main() -> ! {
    let (configurations, init_closure) = configure_device!({
            "gpios": [
                ["PA0", "input", "pull_down", "rising"]
            ]
    });
    let rt =
        Runtime::init(HEAP_START, HEAP_SIZE, &configurations, init_closure).expect("InitError");
    // rt.spawn_task(Task::new(0, example_task()));
    rt.spawn_task(Task::new(0, test_task()));
    rt.run();
}

async fn test_task() {
    let mut gpio = Runtime::get().get_resource_id("event:gpio/pa2").unwrap();
    while let Some(_event) = gpio.read_stream().next().await {
        hprintln!("GPIOEVENT IN MAIN");
    }
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    hprintln!("async number: {}", number);
}
