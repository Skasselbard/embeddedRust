use super::*;
use core::sync::atomic::AtomicUsize;
use core::task::{RawWaker, RawWakerVTable, Waker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskID(usize);

pub struct Task {
    id: TaskID,
    /// TODO: Maybe consider stack pinning:
    /// https://doc.rust-lang.org/stable/std/pin/index.html#projections-and-structural-pinning
    /// as mentioned in phil oppps blog:
    /// https://os.phil-opp.com/async-await/#pinning
    future: Pin<Box<dyn Future<Output = ()>>>,
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
    #[inline]
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
    pub(crate) fn id(&self) -> TaskID {
        self.id
    }
    #[inline]
    pub fn spawn(self) {
        Runtime::get().spawn_task(self)
    }
    #[inline]
    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
    /// moves task into waker
    #[inline]
    pub(crate) fn waker(&self) -> Waker {
        unsafe { Waker::from_raw(raw_waker(self as *const Task as *const ())) }
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
