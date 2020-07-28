pub mod pwm;
pub mod usart;

// reexports of internal types
#[cfg(feature = "stm32f1xx")]
pub use embedded_rust_devices::{Direction, ExtiEvent, Gpio, Pin, PinMode};

// reexports of external types
#[cfg(feature = "stm32f1xx")]
pub type DeviceInterrupt = stm32f1xx_hal::device::Interrupt;
