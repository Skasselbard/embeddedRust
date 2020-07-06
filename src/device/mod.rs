use super::resources::*;

pub mod pwm;
pub mod usart;

// macro export
#[cfg(feature = "stm32f1xx")]
#[macro_use]
pub mod stm32f1xx;

// reexports of internal types
#[cfg(feature = "stm32f1xx")]
pub use stm32f1xx::{ExtiEvent, Gpio, Pin};

// reexports of external types
#[cfg(feature = "stm32f1xx")]
pub type DeviceInterrupt = stm32f1xx_hal::device::Interrupt;
