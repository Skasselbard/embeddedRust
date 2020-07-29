use crate::device::stm32f1xx::{DeviceInterrupt, ExtiEvent};
use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PushError};

pub enum Priority {
    /// priority 0: System Errors, Faults etc
    Error,
    /// priority 1: Important Events and task specific errors
    Critical,
    /// priority 2: Standard non critical event
    Normal,
}

pub(crate) static ERROR_QUEUE: OnceCell<ArrayQueue<Event>> = OnceCell::uninit();
pub(crate) static CRITICAL_QUEUE: OnceCell<ArrayQueue<Event>> = OnceCell::uninit();
pub(crate) static NORMAL_QUEUE: OnceCell<ArrayQueue<Event>> = OnceCell::uninit();

pub fn init(queue_buffer: usize) -> Result<(), conquer_once::TryInitError> {
    ERROR_QUEUE.try_init_once(|| crossbeam_queue::ArrayQueue::new(queue_buffer))?;
    CRITICAL_QUEUE.try_init_once(|| crossbeam_queue::ArrayQueue::new(queue_buffer))?;
    NORMAL_QUEUE.try_init_once(|| crossbeam_queue::ArrayQueue::new(queue_buffer))?;
    Ok(())
}

#[non_exhaustive]
#[derive(Clone)]
pub enum Event {
    DeviceInterrupt(DeviceInterrupt),
    ExternalInterrupt(ExtiEvent),
}

pub fn next() -> Option<Event> {
    pop(Priority::Error)
        .or(pop(Priority::Critical))
        .or(pop(Priority::Normal))
}

/// Next event with given priority if any,
/// None otherwise
fn pop(prio: Priority) -> Option<Event> {
    let queue = match prio {
        Priority::Error => &ERROR_QUEUE,
        Priority::Critical => &CRITICAL_QUEUE,
        Priority::Normal => &NORMAL_QUEUE,
    };
    queue
        .try_get()
        .expect("Uninitialized event queue")
        .pop()
        .ok()
}
/// **Error**
/// if the queue is full
pub fn push(event: Event, prio: Priority) -> Result<(), Event> {
    let queue = match prio {
        Priority::Error => &ERROR_QUEUE,
        Priority::Critical => &CRITICAL_QUEUE,
        Priority::Normal => &NORMAL_QUEUE,
    };
    match queue
        .try_get()
        .expect("Uninitialized event queue")
        .push(event)
    {
        Ok(()) => Ok(()),
        Err(PushError(e)) => Err(e),
    }
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DeviceInterrupt(e) => write!(f, "DeviceInterrupt({:?})", e),
            Event::ExternalInterrupt(i) => write!(f, "ExternalInterrupt"),
        }
    }
}
