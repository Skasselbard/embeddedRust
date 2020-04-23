use super::resources::*;

pub mod pwm;
pub mod usart;

#[cfg(feature = "stm32f1xx")]
#[macro_use]
pub mod stm32f1xx;

#[cfg(feature = "stm32f1xx")]
pub type DeviceInterrupt = stm32f1xx_hal::device::Interrupt;

