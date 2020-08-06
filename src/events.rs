use crate::device::stm32f1xx::{DeviceInterrupt, ExtiEvent};
use crate::RuntimeError;
use alloc::collections::VecDeque;
use cortex_m::interrupt::CriticalSection;

#[derive(Debug)]
pub enum Priority {
    /// priority 0: System Errors, Faults etc
    Error,
    /// priority 1: Important Events and task specific errors
    Critical,
    /// priority 2: Standard non critical event
    Normal,
}

// TODO: prevent heap allocations in interrupts!
// TODO: make it a stream? from futures-util
static mut ERROR_QUEUE: Option<VecDeque<Event>> = None;
static mut CRITICAL_QUEUE: Option<VecDeque<Event>> = None;
static mut NORMAL_QUEUE: Option<VecDeque<Event>> = None;

pub fn init(queue_buffer: usize) -> Result<(), RuntimeError> {
    unsafe {
        if let Some(_) = ERROR_QUEUE {
            return Err(RuntimeError::MultipleInitializations);
        }
        if let Some(_) = CRITICAL_QUEUE {
            return Err(RuntimeError::MultipleInitializations);
        }
        if let Some(_) = NORMAL_QUEUE {
            return Err(RuntimeError::MultipleInitializations);
        }
        ERROR_QUEUE = Some(VecDeque::with_capacity(queue_buffer));
        CRITICAL_QUEUE = Some(VecDeque::with_capacity(queue_buffer));
        NORMAL_QUEUE = Some(VecDeque::with_capacity(queue_buffer));
    }
    Ok(())
}

fn get_queue(prio: Priority) -> &'static mut VecDeque<Event> {
    let queue = unsafe {
        match prio {
            Priority::Error => &mut ERROR_QUEUE,
            Priority::Critical => &mut CRITICAL_QUEUE,
            Priority::Normal => &mut NORMAL_QUEUE,
        }
    };
    match queue {
        Some(inner) => inner,
        None => panic!("uninitialized event queue"),
    }
}

#[non_exhaustive]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Event {
    DeviceInterrupt,
    ExternalInterrupt(ExtiEvent),
}

pub fn next() -> Option<Event> {
    log::trace!("get next event");
    pop(Priority::Error).or(pop(Priority::Critical).or(pop(Priority::Normal)))
}

/// Next event with given priority if any,
/// None otherwise
fn pop(prio: Priority) -> Option<Event> {
    get_queue(prio).pop_front()
}
/// **Error**
/// if the queue is full
pub fn push(event: Event, prio: Priority, _cs: &CriticalSection) {
    log::trace!("push event {:?} - {:?}", event, prio);
    get_queue(prio).push_back(event)
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DeviceInterrupt => write!(f, "DeviceInterrupt"),
            Event::ExternalInterrupt(i) => write!(f, "ExternalInterrupt({:?})", i),
        }
    }
}
