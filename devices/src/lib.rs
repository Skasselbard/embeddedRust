#![no_std]
#![feature(const_fn)]
#![feature(const_btree_new)]

extern crate alloc;

pub mod events;
pub mod resources;
mod stm32f1xx;

pub use stm32f1xx::*;

// reexports of external types
// #[cfg(feature = "stm32f1xx")]
pub type DeviceInterrupt = stm32f1xx_hal::device::Interrupt;
