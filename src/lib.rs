#![no_std]
#![feature(const_btree_new)]
#![feature(option_expect_none)]
#![feature(const_fn)]
extern crate alloc;

#[macro_use]
pub mod device;

mod executor;
pub mod schemes;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use embedded_rust_devices::events;
use embedded_rust_devices::resources::{Resource, ResourceID};
use embedded_rust_devices::ComponentConfiguration;
use nom_uri::Uri;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Runtime {
    // resource_ids: BTreeMap<Uri<>, ResourceID)>,
    resources: Vec<Box<dyn Resource>>,
    executor: executor::Executor,
    //TODO: implement callbacks (observer pattern?)
    // event_callbacks: BTreeMap<Event, Task>
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    MultipleInitializations,
    ResourceNotFound,
    TaskQueueIsFull,
}

impl Runtime {
    pub fn init<F: FnOnce()>(
        heap_bottom: usize,
        heap_size: usize,
        max_events_per_prio: usize,
        resource_configuration: &[ComponentConfiguration],
        init_closure: F,
    ) -> Result<Self, RuntimeError> {
        Self::init_heap(heap_bottom, heap_size);
        events::init(max_events_per_prio)
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        let mut rt = Self {
            resources: Vec::with_capacity(resource_configuration.len()),
            executor: executor::Executor::new(),
        };
        rt.configure(resource_configuration);
        init_closure();
        Ok(rt)
    }
    pub fn get_resource(&mut self, id: &ResourceID) -> Result<&mut dyn Resource, RuntimeError> {
        Ok(self
            .resources
            .get_mut(id.0 as usize)
            .expect("Resource id not found in vector")
            .as_mut())
    }
    pub fn get_resource_id<'uri>(&mut self, uri: &'uri Uri) -> Result<ResourceID, RuntimeError> {
        let mut id = None;
        let mut buffer = String::new();
        for i in 0..self.resources.len() {
            if &self.resources[i].to_uri(&mut buffer) == uri {
                id = Some(ResourceID(i as u8));
                break;
            }
        }
        match id {
            Some(id) => Ok(id),
            None => Err(RuntimeError::ResourceNotFound),
        }
    }
    pub fn add_resource(&mut self, resource: Box<dyn Resource>) -> ResourceID {
        let id = self.resources.len();
        self.resources.push(resource);
        ResourceID(id as u8)
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
            //FIXME: event handling probably belongs in the executor
            while let Some(event) = events::next() {
                self.handle_event(event);
            }
            // TODO: ? waker checken ?
            // TODO: do something  with the result!
            self.executor.run();
            cortex_m::asm::wfi(); // safe power till next interrupt
        }
    }
    pub fn spawn_task(&mut self, task: Task, priority: usize) {
        self.executor.spawn(task, priority)
    }
    fn handle_event(&self, event: events::Event) {
        match event {
            _ => unimplemented!(),
        }
    }
    fn configure(&mut self, configurations: &[ComponentConfiguration]) {
        for configuration in configurations {
            let resource: Box<dyn Resource> = match configuration {
                ComponentConfiguration::Gpio(gpio) => Box::new(gpio.clone()),
                _ => unimplemented!(),
            };
            self.add_resource(resource);
        }
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
