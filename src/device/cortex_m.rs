use core::panic::PanicInfo;
use cortex_m_rt::exception;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // log::error!("panic: {}", info);
    cortex_m_semihosting::hprintln!("panic: {}", info).unwrap();
    cortex_m::interrupt::disable();
    loop {}
}

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // prints the exception frame as a panic message
    panic!("HardFault: {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    cortex_m_semihosting::hprintln!("IRQn = {}", irqn).unwrap();
}

/// Creates a new heap with the given bottom and size. The bottom address must be
/// valid and the memory in the [heap_bottom, heap_bottom + heap_size) range must not
/// be used for anything else. This function is unsafe because it can cause undefined
/// behavior if the given address is invalid.
pub(crate) fn init_heap(heap_bottom: usize, heap_size: usize) {
    unsafe { ALLOCATOR.lock().init(heap_bottom, heap_size) };
}
