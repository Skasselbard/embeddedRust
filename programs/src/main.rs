#![no_main]
#![no_std]

extern crate alloc;

// use cortex_m_semihosting::hprintln;

use cortex_m_rt::entry;
use embedded_rust::io::{AsyncReadExt, AsyncWriteExt};
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
                ["PA0", "input", "pull_up", "falling"],
                ["PC13", "output", "push_pull"]
            ]
        }
    })
]
struct BluePill;
#[entry]
fn main() -> ! {
    BluePill::init();
    Task::new(test_task()).spawn();
    BluePill::run();
}

pub async fn test_task() {
    let mut button_events = BluePill::get_resource("event:gpio/pa0").unwrap();
    let mut led = BluePill::get_resource("digital:gpio/pc13").unwrap();
    let mut led_state = false;
    let mut buf = [0; 1];
    while let Ok(_count) = button_events.read(&mut buf).await {
        led_state = !led_state;
        led.write(&[led_state as u8]).await.unwrap();
    }
}
