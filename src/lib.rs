#![no_std]
#![feature(option_expect_none)]
#![feature(const_fn)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]

extern crate alloc;

#[macro_use]
pub mod device;

pub mod events;
mod executor;
mod logging;
pub mod resources;
pub mod schemes;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use device::stm32f1xx::ComponentConfiguration;
use futures::{Stream, StreamExt};
use log::{trace, Level};
use nom_uri::Uri;
use resources::{Resource, ResourceError, ResourceID};

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    cortex_m::interrupt::disable();
    loop {}
}

pub struct Task {
    priority: usize,
    /// TODO: Maybe consider stack pinning:
    /// https://doc.rust-lang.org/stable/std/pin/index.html#projections-and-structural-pinning
    /// as mentioned in phil oppps blog:
    /// https://os.phil-opp.com/async-await/#pinning
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Runtime {
    // resource_ids: BTreeMap<Uri<>, ResourceID)>,
    resources: Vec<Box<dyn Resource>>,
    executor: executor::Executor,
    //TODO: implement callbacks (observer pattern?)
    // event_callbacks: BTreeMap<Event, Task>
    resource_cache: BTreeMap<String, ResourceID>,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    UninitializedAccess,
    MultipleInitializations,
    ResourceNotFound,
    TaskQueueIsFull,
    UriParseError,
}

impl Runtime {
    pub fn init<F: FnOnce()>(
        heap_bottom: usize,
        heap_size: usize,
        max_events_per_prio: usize,
        resource_configuration: &[ComponentConfiguration],
        init_closure: F,
    ) -> Result<&'static mut Self, RuntimeError> {
        let inner = Self::get_inner();
        if let Some(_) = inner {
            return Err(RuntimeError::MultipleInitializations);
        };
        Self::init_heap(heap_bottom, heap_size);
        events::init(max_events_per_prio)
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        let mut rt = Self {
            resources: Vec::with_capacity(resource_configuration.len()),
            executor: executor::Executor::new(),
            resource_cache: BTreeMap::new(),
        };
        rt.configure(resource_configuration);
        logging::init().expect("log initialization failed");
        inner.replace(rt);
        // init_closure();
        Ok(Self::get())
    }
    fn get_inner() -> &'static mut Option<Self> {
        static mut RUNTIME: Option<Runtime> = None;
        unsafe { &mut RUNTIME }
    }
    pub fn get() -> &'static mut Runtime {
        Self::get_inner().as_mut().expect("uninitialized runtime")
    }
    fn get_resource(&mut self, id: &ResourceID) -> Result<&mut dyn Resource, RuntimeError> {
        Ok(self
            .resources
            .get_mut(id.0 as usize)
            .expect("Resource id not found in vector")
            .as_mut())
    }
    pub fn get_resource_id(&mut self, uri: &str) -> Result<ResourceID, RuntimeError> {
        use core::convert::TryFrom;
        match self.resource_cache.get(uri.into()) {
            Some(id) => Ok(id.clone()),
            None => {
                let mut id = None;
                let mut buffer = String::new();
                for i in 0..self.resources.len() {
                    let parsed_uri = Uri::try_from(uri).or(Err(RuntimeError::UriParseError))?;
                    if self.resources[i].to_uri(&mut buffer) == parsed_uri {
                        id = Some(ResourceID(i as u8));
                        break;
                    }
                }
                match id {
                    Some(id) => {
                        self.resource_cache.insert(uri.into(), id.clone());
                        Ok(id)
                    }
                    None => Err(RuntimeError::ResourceNotFound),
                }
            }
        }
    }
    fn add_resource(&mut self, resource: Box<dyn Resource>) -> ResourceID {
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
        //TODO: enable interrupts here
        trace!("run");
        loop {
            // TODO: ? waker checken ?
            // TODO: do something  with the result!
            self.executor.run();
            trace!("sleep");
            cortex_m::asm::wfi(); // safe power till next interrupt
        }
    }
    pub fn spawn_task(&mut self, task: Task) {
        self.executor.spawn(task)
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
    /// zero is highest priority
    pub fn new(priority: usize, future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
            priority,
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[allow(unused)]
impl ResourceID {
    fn read_stream(&mut self) -> impl Stream<Item = u8> {
        use futures::stream::poll_fn;
        let id = *self;
        poll_fn(move |cx| Runtime::get().get_resource(&id).unwrap().read_next(cx))
        // struct S {
        //     id: ResourceID,
        // };
        // impl Stream for S {
        //     type Item = u8;
        //     fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        //         Runtime::get().get_resource(&self.id).unwrap().read_next(cx)
        //     }
        // }
        // S { id: *self }
    }
    async fn write(
        &mut self,
        mut stream: impl StreamExt<Item = u8> + Unpin,
    ) -> Result<(), ResourceError> {
        use futures::future::poll_fn;

        let res = Runtime::get().get_resource(self).unwrap();
        while let Some(byte) = stream.next().await {
            poll_fn(|cx| (res.write_next(cx, byte))).await?
        }
        Ok(())
    }
    async fn seek(&mut self, pos: usize) -> Result<(), ResourceError> {
        use futures::future::poll_fn;
        poll_fn(|cx| Runtime::get().get_resource(self).unwrap().seek(cx, pos)).await
    }
    fn to_uri<'uri>(&self, buffer: &'uri mut str) -> Uri<'uri> {
        Runtime::get().get_resource(self).unwrap().to_uri(buffer)
    }
}

impl Eq for Task {}
impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(&self.priority, &other.priority)
    }
}
impl Ord for Task {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
