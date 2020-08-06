use super::*;
use crate::events::Event;
use alloc::collections::{BTreeMap, BinaryHeap};
use core::task::{RawWaker, Waker};
use log::trace;

pub struct Executor {
    task_queue: BinaryHeap<Task>,
    event_tasks: BTreeMap<Event, Waker>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_queue: BinaryHeap::new(),
            event_tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push(task);
    }
    pub fn run(&mut self) {
        trace!("executor");
        loop {
            if let Some(event) = events::next() {
                self.handle_event(event);
                continue; // first handle all events
            }
            if let Some(mut task) = self.task_queue.pop() {
                let waker = dummy_waker();
                let mut context = Context::from_waker(&waker);
                match task.poll(&mut context) {
                    Poll::Ready(()) => {} // task done
                    Poll::Pending => self.spawn(task),
                }
                continue; // test for additional events and task before termination
            }
            break;
        }
    }
    pub fn register_event_task(&mut self, event: events::Event, task: Task) {
        unimplemented!();
        // self.event_tasks.insert(event, task);
    }

    fn handle_event(&self, event: events::Event) {
        trace!("event: {:?}", event);
        // if let Some(task) = self.event_tasks.get(&event) {
        //     let waker = dummy_waker();
        //     let mut context = Context::from_waker(&waker);
        //     match task.poll(&mut context) {
        //         Poll::Ready(()) => {} // task done
        //         Poll::Pending => {}   //self.spawn(task),
        //     }
        // }
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
