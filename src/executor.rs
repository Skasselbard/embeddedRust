use super::*;
use crate::events::Event;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::task::Waker;
use device::handle_exti_event;
use heapless::spsc::Queue;
use task::TaskID;

pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    /// T = TaskID, max length = 256
    task_queue: Queue<TaskID, 256>,
    /// If an event is fired, these wakers requeue the corresponding tasks
    event_wakers: BTreeMap<Event, Vec<Waker>>,
}

impl Executor {
    #[inline]
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: unsafe { Queue::new() },
            event_wakers: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.enqueue(task.id()).expect("task queue full");
        self.tasks.insert(task.id(), task);
    }
    #[inline]
    pub(crate) fn reque(&mut self, task_id: TaskID) {
        self.task_queue.enqueue(task_id).expect("task queue full")
    }
    pub fn run(&mut self) {
        loop {
            self.wake_tasks();
            if let Some(task_id) = self.task_queue.dequeue() {
                let task = self.tasks.get_mut(&task_id).expect("missing task");
                let waker = task.waker();
                let mut context = Context::from_waker(&waker);
                match task.poll(&mut context) {
                    Poll::Ready(()) => {} // task done
                    Poll::Pending => {}
                }
                continue; // test for additional events and task before termination
            }
            break;
        }
    }
    #[inline]
    fn wake_tasks(&mut self) {
        while let Some(event) = events::next() {
            if let Some(wakers) = self.event_wakers.get_mut(&event) {
                // only do event specific things if someone actuall is expecting events
                // log::info!("E");
                Self::handle_event(&event);
                while let Some(waker) = wakers.pop() {
                    waker.wake_by_ref()
                }
            }
        }
    }

    /// Trigger event specific behaviour
    #[inline]
    fn handle_event(event: &Event) {
        match event {
            Event::ExternalInterrupt(exti_event) => handle_exti_event(exti_event),
            Event::DeviceInterrupt => {}
            Event::SerialEvent(_id) => {}
        }
    }

    pub(crate) fn register_waker(&mut self, trigger: &Event, waker: &Waker) {
        let wakers = if let Some(w) = self.event_wakers.get_mut(trigger) {
            w
        } else {
            self.event_wakers.insert(trigger.clone(), Vec::new());
            self.event_wakers.get_mut(trigger).unwrap()
        };
        wakers.push(waker.clone());
    }
}
