use crate::device::stm32f1xx::ExtiEvent;
use alloc::collections::VecDeque;
use core::ops::DerefMut;
use cortex_m::interrupt::CriticalSection;
use once_cell::unsync::Lazy;

pub(crate) fn get_queue() -> &'static mut VecDeque<Event> {
    // TODO: prevent heap allocations in interrupts!
    // TODO: make it a stream? from futures-util
    // static mut EVENT_QUEUE: Lazy<VecDeque<Event>> = Lazy::new(|| VecDeque::with_capacity(10));
    // unsafe { EVENT_QUEUE.deref_mut() }
    static mut EVENT_QUEUE: Option<VecDeque<Event>> = None;
    unsafe {
        if let None = EVENT_QUEUE {
            EVENT_QUEUE = Some(VecDeque::with_capacity(10));
        }
        EVENT_QUEUE.as_mut().unwrap()
    }
}

#[non_exhaustive]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Event {
    DeviceInterrupt,
    ExternalInterrupt(ExtiEvent),
}

//TODO: add critical section?
pub fn next() -> Option<Event> {
    log::trace!("get next event");
    get_queue().pop_front()
}

pub fn push(event: Event, _cs: &CriticalSection) {
    log::trace!("push event {:?}", event);
    get_queue().push_back(event)
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DeviceInterrupt => write!(f, "DeviceInterrupt"),
            Event::ExternalInterrupt(i) => write!(f, "ExternalInterrupt({:?})", i),
        }
    }
}
