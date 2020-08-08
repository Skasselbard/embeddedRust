use super::*;
use crate::events::Event;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use core::task::{RawWaker, RawWakerVTable, Waker};
use heapless::consts::*;
use heapless::spsc::{Queue, SingleCore};
use log::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskID(usize);

pub struct Task {
    id: TaskID,
    /// TODO: Maybe consider stack pinning:
    /// https://doc.rust-lang.org/stable/std/pin/index.html#projections-and-structural-pinning
    /// as mentioned in phil oppps blog:
    /// https://os.phil-opp.com/async-await/#pinning
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    task_queue: Queue<TaskID, U256, u8, SingleCore>,
    /// If an event is fired, these wakers requeue the corresponding tasks
    event_wakers: BTreeMap<Event, Vec<Waker>>,
}
fn raw_waker(task: *const ()) -> RawWaker {
    fn clone(task: *const ()) -> RawWaker {
        raw_waker(task)
    }
    fn wake(task: *const ()) {
        wake_by_ref(task)
    }
    fn wake_by_ref(task: *const ()) {
        unsafe { Runtime::get().executor.reque((*(task as *const Task)).id) }
    }
    fn drop(_task: *const ()) {}

    let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(task, vtable)
}

impl TaskID {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        TaskID(NEXT_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

impl Task {
    /// zero is highest priority
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskID::new(),
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
    // moves task into waker
    fn waker(&self) -> Waker {
        unsafe { Waker::from_raw(raw_waker(self as *const Task as *const ())) }
    }
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: unsafe { Queue::u8_sc() },
            event_wakers: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        trace!("enqueue");
        self.task_queue.enqueue(task.id).expect("task queue full");
        trace!("insert");
        self.tasks.insert(task.id, task);
        trace!("end");
    }
    fn reque(&mut self, task_id: TaskID) {
        self.task_queue.enqueue(task_id).expect("task queue full")
    }
    pub fn run(&mut self) {
        trace!("executor");
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

    fn wake_tasks(&mut self) {
        //TODO: remove duplicates
        for event in events::get_queue().into_iter() {
            if let Some(wakers) = self.event_wakers.get(event) {
                trace!("event: {:?}", event);
                for waker in wakers.into_iter() {
                    waker.wake_by_ref()
                }
                // TODO: test and delete
                assert!(wakers.is_empty());
            }
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

impl Eq for Task {}
impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(&self.id, &other.id)
    }
}
// impl Ord for Task {
//     fn cmp(&self, other: &Self) -> core::cmp::Ordering {
//         self.priority.cmp(&other.priority)
//     }
// }
// impl PartialOrd for Task {
//     fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
