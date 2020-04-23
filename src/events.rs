use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PushError};
use crate::{ResourceID, DeviceInterrupt};

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

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum Event {
    ResourceEvent(DeviceInterrupt),
}

/// Next event if any,
/// None otherwise
pub fn pop(prio: Priority) -> Option<Event> {
    let queue = match prio {
        Priority::Error => &ERROR_QUEUE,
        Priority::Critical => &CRITICAL_QUEUE,
        Priority::Normal => &NORMAL_QUEUE,
    };
    queue.try_get().expect("Uninitialized event queue").pop().ok()
}
/// **Error**
/// if the queue is full
pub fn push(event: Event, prio: Priority) -> Result<(), Event> {
    let queue = match prio {
        Priority::Error => &ERROR_QUEUE,
        Priority::Critical => &CRITICAL_QUEUE,
        Priority::Normal => &NORMAL_QUEUE,
    };
    match queue.try_get().expect("Uninitialized event queue").push(event) {
        Ok(()) => Ok(()),
        Err(PushError(e)) => Err(e),
    }
}
