#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use futures::StreamExt;

use cortex_m_rt::entry;
use embedded_rust::Task;
use embedded_rust_macros::*;
#[
    device_config({
        "Stm32f1xx":{
            "sys": {
                "heap_size": [10, "kb"],
                "sys_clock": [36, "mhz"]
            },
            "gpios": [
                ["PA0", "input", "pull_down", "rising"],
                ["PA1", "output", "push_pull"],
                ["PA2", "input", "pull_down", "falling"]
            ]
        }
    })
]
struct BluePill;
#[entry]
fn main() -> ! {
    BluePill::init();
    Task::new(example_task()).spawn();
    Task::new(test_task()).spawn();
    BluePill::run();
}

pub async fn test_task() {
    let mut gpio = BluePill::get_resource("event:gpio/pa0").unwrap();
    while let Some(_event) = gpio.read_stream().next().await {
        hprintln!("GPIOEVENT IN MAIN {}", _event);
    }
}

pub async fn async_number() -> u32 {
    42
}

pub async fn example_task() {
    let number = async_number().await;
    hprintln!("async number: {}", number);
}
