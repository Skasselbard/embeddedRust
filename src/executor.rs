use super::*;
use core::task::{RawWaker, Waker};
use heapless::binary_heap::{BinaryHeap, Min};
use heapless::consts::*;

pub struct Executor {
    /// lower value means higher priority
    task_queue: BinaryHeap<HeapElement, U32, Min>,
}

struct HeapElement {
    priority: usize,
    task: Task,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_queue: BinaryHeap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task, priority: usize) -> Result<(), RuntimeError> {
        match self.task_queue.push(HeapElement { priority, task }) {
            Ok(_) => Ok(()),
            Err(_) => Err(RuntimeError::TaskQueueIsFull),
        }
    }
    pub fn run(&mut self) {
        while let Some(HeapElement { priority, mut task }) = self.task_queue.pop() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.spawn(task, priority).expect("task requeue failed"),
            }
        }
    }
}

use core::task::RawWakerVTable;
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

impl Eq for HeapElement {}
impl PartialEq for HeapElement {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(&self.task, &other.task)
    }
}
impl Ord for HeapElement {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl PartialOrd for HeapElement {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
