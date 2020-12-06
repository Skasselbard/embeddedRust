use crate::device::ExtiEvent;
use cortex_m::interrupt::CriticalSection;
use heapless::consts::U32;
use heapless::spsc::{Queue, SingleCore};

// TODO: multicore with feature
#[inline]
pub(crate) fn get_queue() -> &'static mut Queue<Event, U32, u8, SingleCore> {
    static mut EVENT_QUEUE: Option<Queue<Event, U32, u8, SingleCore>> = None;
    unsafe {
        if let None = EVENT_QUEUE {
            EVENT_QUEUE = Some(Queue::u8_sc());
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
#[inline]
pub fn next() -> Option<Event> {
    get_queue().dequeue()
}

#[inline]
pub fn push(event: Event, _cs: &CriticalSection) {
    get_queue().enqueue(event).expect("filled event_queue")
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DeviceInterrupt => write!(f, "DeviceInterrupt"),
            Event::ExternalInterrupt(i) => write!(f, "ExternalInterrupt({:?})", i),
        }
    }
}
