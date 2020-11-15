mod cortex_m;

#[cfg(feature = "stm32f1xx")]
mod stm32f1xx;
#[cfg(feature = "stm32f1xx")]
pub(crate) use self::cortex_m::init_heap;
#[cfg(feature = "stm32f1xx")]
use stm32f1xx as dev;

pub type ExtiEvent = dev::ExtiEvent;
pub type Channel = dev::Channel;
pub type Port = dev::Port;
pub type SerialID = dev::SerialID;

/// Should return the start of the heap allocation
/// In stm32f1 it startts at the data segment .uninit after .bss
#[inline]
pub fn heap_bottom() -> usize {
    dev::heap_bottom()
}

/// Wait for the next event
/// For ARM-Cortex-M3 cores this is the wfe (wait for event) instruction
#[inline]
pub fn sleep() {
    dev::sleep()
}

#[inline]
pub fn handle_exti_event(event: &ExtiEvent) {
    dev::handle_exti_event(event)
}
