use crate::{
    device::{ExtiEvent, SerialID},
    resources::serial::SerialDirection,
};
use heapless::spsc::{Queue, SingleCore};

// TODO: multicore with feature
#[inline]
pub(crate) fn get_queue() -> &'static mut Queue<Event, heapless::consts::U32, u8, SingleCore> {
    // TODO: Make it Nonblocking QUEUE like BBQUEUE
    static mut EVENT_QUEUE: Option<Queue<Event, heapless::consts::U32, u8, SingleCore>> = None;
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
    DeviceInterrupt,              //TODO: Probably not nedded
    ExternalInterrupt(ExtiEvent), //TODO: Make it a GPIO Event
    SerialEvent(SerialID),
}

//TODO: add critical section?
#[inline]
pub fn next() -> Option<Event> {
    get_queue().dequeue()
}

#[inline]
pub fn push(event: Event) {
    get_queue().enqueue(event).expect("filled event_queue")
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DeviceInterrupt => write!(f, "DeviceInterrupt"),
            Event::ExternalInterrupt(i) => write!(f, "ExternalInterrupt({:?})", i),
            Event::SerialEvent(id) => {
                write!(f, "SerialEvent({:?})", id)
            }
        }
    }
}
