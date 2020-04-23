#![no_std]
extern crate alloc;

#[macro_use]
extern crate nb;

#[macro_use]
pub mod device;

pub(crate) mod events;
mod executor;
pub mod io;
pub mod resources;
pub mod schemes;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
pub use device::DeviceInterrupt;
use heapless::FnvIndexMap as HashMap;
use heapless::{consts::*, Vec};
use nom_uri::Uri;
pub use resources::{Resource, ResourceID};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

/// Max allowed elements in resource list.
/// ['ResourceID'] only backs values up to 255
pub type ResourceCount = U32;

pub struct Runtime<'device> {
    resource_ids: HashMap<Uri<'device>, ResourceID, ResourceCount>,
    resources: Vec<&'device mut dyn Resource, ResourceCount>,
    associated_interrupts: Vec<alloc::vec::Vec<DeviceInterrupt>, ResourceCount>,
    executor: executor::Executor,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    MultipleInitializations,
    ResourceNotFound,
    TaskQueueIsFull,
}

impl<'device> Runtime<'device> {
    pub fn init(
        heap_bottom: usize,
        heap_size: usize,
        max_events_per_prio: usize,
    ) -> Result<Self, RuntimeError> {
        Self::init_heap(heap_bottom, heap_size);
        events::ERROR_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        events::CRITICAL_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        events::NORMAL_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        Ok(Self {
            resource_ids: HashMap::<_, _, ResourceCount>::new(),
            resources: Vec::<_, ResourceCount>::new(),
            associated_interrupts: Vec::<_, ResourceCount>::new(),
            executor: executor::Executor::new(),
        })
    }
    pub fn get_resource<'uri: 'device>(
        &mut self,
        id: ResourceID,
    ) -> Result<(ResourceID, &mut dyn Resource), RuntimeError> {
        Ok((
            id,
            *self
                .resources
                .get_mut(id.0 as usize)
                .expect("Resource id not found in vector"),
        ))
    }
    pub fn get_resource_from_uri<'uri: 'device>(
        &mut self,
        uri: &'uri Uri,
    ) -> Result<(ResourceID, &mut dyn Resource), RuntimeError> {
        let id = *match self.resource_ids.get_mut(uri) {
            Some(id) => id,
            None => return Err(RuntimeError::ResourceNotFound),
        };
        self.get_resource(id)
    }
    pub fn add_resource<'uri: 'device>(
        &mut self,
        uri: Uri<'uri>,
        resource: &'device mut dyn Resource,
    ) -> ResourceID {
        let id = self.resources.len();
        self.resources
            .push(resource)
            .unwrap_or_else(|_| panic!("Resource array full"));
        self.resource_ids
            .insert(uri, ResourceID(id as u8))
            .unwrap_or_else(|_| panic!("Resource array full"));
        self.associated_interrupts
            .push(alloc::vec::Vec::with_capacity(0))
            .unwrap_or_else(|_| panic!("Resource array full"));
        ResourceID(id as u8)
    }
    pub fn associate_interrupt(
        &mut self,
        resource: ResourceID,
        interrupt: DeviceInterrupt,
    ) -> Result<(), RuntimeError> {
        match self.associated_interrupts.get_mut(resource.0 as usize) {
            Some(vector) => {
                vector.push(interrupt);
                Ok(())
            }
            None => Err(RuntimeError::ResourceNotFound),
        }
    }
    /// Creates a new heap with the given bottom and size. The bottom address must be
    /// valid and the memory in the [heap_bottom, heap_bottom + heap_size) range must not
    /// be used for anything else. This function is unsafe because it can cause undefined
    /// behavior if the given address is invalid.
    fn init_heap(heap_bottom: usize, heap_size: usize) {
        unsafe { ALLOCATOR.lock().init(heap_bottom, heap_size) };
    }
    pub fn run(&mut self) -> ! {
        loop {
            // interrupts checken -> device spezifisch
            // ? waker checken ?
            // TODO: do something  eith the result!
            self.executor.run();
            cortex_m::asm::wfi(); // safe power till next interrupt
        }
    }
    pub fn spawn_task(&mut self, task: Task, priority: usize) -> Result<(), RuntimeError> {
        self.executor.spawn(task, priority)
    }
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
