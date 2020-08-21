#[macro_use]
mod usart;
mod gpio;
mod pwm;
pub use gpio::*;
pub use pwm::*;
pub use usart::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub enum ExtiEvent {
    Gpio(super::Pin),
    Pvd,
    RtcAlarm,
    UsbWakeup,
    EthernetWakeup,
}

/// The heap starts after the data segments of static values (.data and .bss)
/// #[link_section] places the annotated static directly at the given data segment.
/// We can use the adress of this static to determine the start of the heap
/// if we use the .uninit segment (unoccupied data after .bss) as section.
/// See the [cortex-m-rt documentation](https://docs.rs/cortex-m-rt/0.6.12/cortex_m_rt/#uninitialized-static-variables) and [link section reference](https://doc.rust-lang.org/reference/abi.html#the-link_section-attribute) for mor information
#[inline]
pub fn heap_bottom() -> usize {
    #[link_section = ".uninit"]
    static HEAP_BOTTOM: usize = 0;
    &HEAP_BOTTOM as *const usize as usize
}
#[inline]
pub fn sleep() {
    cortex_m::asm::wfe()
}
