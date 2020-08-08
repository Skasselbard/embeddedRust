#![no_std]
#![feature(const_fn)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
pub mod device;

pub mod events;
pub mod resources;
pub mod schemes;

pub use executor::Task;

mod executor;
mod logging;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
// use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::task::Waker;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use cortex_m_rt::exception;
use device::ComponentConfiguration;
use events::Event;
use futures::StreamExt;
use heapless::consts::*;
use heapless::Vec;
use log::trace;
use nom_uri::Uri;
use resources::{Resource, ResourceError, ResourceID};

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // log::error!("panic: {}", info);
    cortex_m_semihosting::hprintln!("panic: {}", info);
    cortex_m::interrupt::disable();
    loop {}
}

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // prints the exception frame as a panic message
    panic!("HardFault: {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    cortex_m_semihosting::hprintln!("IRQn = {}", irqn);
}

pub struct Runtime {
    static_resources: Vec<&'static mut dyn Resource, U64>,
    executor: executor::Executor,
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
        heap_size: usize,
        resource_configuration: &[ComponentConfiguration],
        init_closure: F,
    ) -> Result<&'static mut Self, RuntimeError> {
        let inner = Self::get_inner();
        if let Some(_) = inner {
            return Err(RuntimeError::MultipleInitializations);
        };
        Self::init_heap(device::heap_bottom(), heap_size);
        inner.replace(Self {
            static_resources: Vec::new(),
            executor: executor::Executor::new(),
        });
        let rt = Self::get();
        rt.configure(resource_configuration);
        logging::init().expect("log initialization failed");
        trace!("test");
        init_closure();
        Ok(rt)
    }
    fn get_inner() -> &'static mut Option<Self> {
        static mut RUNTIME: Option<Runtime> = None;
        unsafe { &mut RUNTIME }
    }
    pub fn get() -> &'static mut Runtime {
        Self::get_inner().as_mut().expect("uninitialized runtime")
    }
    fn get_resource(&'static mut self, id: &ResourceID) -> &'static mut dyn Resource {
        *self
            .static_resources
            .get_mut(id.0 as usize)
            .expect("Resource id not found in vector")
    }
    pub fn get_resource_id(&mut self, uri: &str) -> Result<ResourceID, RuntimeError> {
        use core::convert::TryFrom;
        let mut buffer = String::new();
        for i in 0..self.static_resources.len() {
            let parsed_uri = Uri::try_from(uri).or(Err(RuntimeError::UriParseError))?;
            if self.static_resources[i].to_uri(&mut buffer) == parsed_uri {
                return Ok(ResourceID(i as u8));
            }
        }
        Err(RuntimeError::ResourceNotFound)
    }
    fn add_resource(&mut self, resource: &'static mut dyn Resource) -> ResourceID {
        let id = self.static_resources.len();
        self.static_resources.push(resource);
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
            self.executor.run();
            trace!("sleep");
            //TODO: move to device
            cortex_m::asm::wfe(); // safe power till cpu event
        }
    }
    pub fn spawn_task(&mut self, task: Task) {
        trace!("spawn");
        self.executor.spawn(task);
        trace!("endspawn");
    }
    fn configure(&mut self, configurations: &[ComponentConfiguration]) {
        for configuration in configurations {
            let resource: &dyn Resource = match configuration {
                ComponentConfiguration::Gpio(gpio) => gpio,
                _ => unimplemented!(),
            };
            // TODO: maybe cloning is somehow possible
            let resource = unsafe { &mut *(resource as *const dyn Resource as *mut dyn Resource) };
            self.add_resource(resource);
        }
    }
    pub(crate) fn register_waker(&mut self, trigger: &Event, waker: &Waker) {
        self.executor.register_waker(trigger, waker)
    }
}

#[allow(unused)]
impl ResourceID {
    pub fn read_stream(&mut self) -> impl StreamExt<Item = u8> {
        use futures::stream::poll_fn;
        let id = *self;
        poll_fn(move |cx| Runtime::get().get_resource(&id).read_next(cx))
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
    pub async fn write(
        &mut self,
        mut stream: impl StreamExt<Item = u8> + Unpin,
    ) -> Result<(), ResourceError> {
        use futures::future::poll_fn;

        let res = Runtime::get().get_resource(self);
        while let Some(byte) = stream.next().await {
            poll_fn(|cx| (res.write_next(cx, byte))).await?
        }
        Ok(())
    }
    pub async fn seek(&mut self, pos: usize) -> Result<(), ResourceError> {
        use futures::future::poll_fn;
        poll_fn(|cx| Runtime::get().get_resource(self).seek(cx, pos)).await
    }
    pub fn to_uri<'uri>(&self, buffer: &'uri mut str) -> Uri<'uri> {
        Runtime::get().get_resource(self).to_uri(buffer)
    }
}
