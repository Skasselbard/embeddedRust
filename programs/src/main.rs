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
            ],
            "pwm":[{
                "timer":    "Tim2", 
                "pins":     ["PA1"], 
                "frequency":[10,"khz"]
            }]
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

macro_rules! to_target_endianess {
    ($int:expr) => {
        if cfg!(target_endian = "big") {
            $int.to_be_bytes()
        } else {
            $int.to_le_bytes()
        }
    };
}

enum Level {
    Full,
    High,
    Half,
    Low,
    Off,
}

struct Brightness {
    level: Level,
}

impl Brightness {
    fn next(&mut self) -> f32 {
        match self.level {
            Level::Full => {
                self.level = Level::High;
                0.75f32
            }
            Level::High => {
                self.level = Level::Half;
                0.5f32
            }
            Level::Half => {
                self.level = Level::Low;
                0.25f32
            }
            Level::Low => {
                self.level = Level::Off;
                0.0f32
            }
            Level::Off => {
                self.level = Level::Full;
                1.0f32
            }
        }
    }
}

pub async fn test_task() {
    let mut button_events = BluePill::get_resource("event:gpio/pa0").unwrap();
    let mut led = BluePill::get_resource("digital:gpio/pc13").unwrap();
    let mut brightness = Brightness { level: Level::Off };
    let mut pwm = BluePill::get_resource("analog:pwm/pa1").unwrap();
    pwm.write(&to_target_endianess!(brightness.next()))
        .await
        .unwrap();
    let mut led_state = false;
    let mut buf = [0; 1];
    while let Ok(_count) = button_events.read(&mut buf).await {
        led_state = !led_state;
        led.write(&[led_state as u8]).await.unwrap();
        pwm.write(&to_target_endianess!(brightness.next()))
            .await
            .unwrap();
    }
}
