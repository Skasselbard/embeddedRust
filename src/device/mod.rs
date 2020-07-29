pub mod pwm;
pub mod usart;

// reexports of internal types
#[cfg(feature = "stm32f1xx")]
pub use embedded_rust_devices::{Direction, ExtiEvent, Gpio, Pin, PinMode};

