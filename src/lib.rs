#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

macro_rules! to_target_endianess {
    ($int:expr) => {
        if cfg!(target_endian = "big") {
            $int.to_be_bytes()
        } else {
            $int.to_le_bytes()
        }
    };
}

macro_rules! from_target_endianess {
    ($int_type:ty, $array:expr) => {{
        use core::convert::TryInto;
        match $array.try_into() {
            Ok(value) => Ok(if cfg!(target_endian = "big") {
                <$int_type>::from_be_bytes(value)
            } else {
                <$int_type>::from_le_bytes(value)
            }),
            Err(e) => Err(e),
        }
    }};
}

mod executor;
mod logging;
mod task;
mod utilities;

#[macro_use]
pub mod device;

pub mod events;
pub mod io;
pub mod resources;
pub mod schemes;

pub use task::Task;

use alloc::boxed::Box;
use core::task::Waker;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use events::Event;
use resources::{Resource, ResourceError, ResourceID, Resources};

pub struct Runtime {
    resources: Resources,
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
    pub fn init(
        heap_size: usize,
        sys: &'static mut [&'static mut dyn Resource],
        input_pins: &'static mut [&'static mut dyn Resource],
        output_pins: &'static mut [&'static mut dyn Resource],
        pwm: &'static mut [&'static mut dyn Resource],
        channels: &'static mut [&'static mut dyn Resource],
        serials: &'static mut [&'static mut dyn Resource],
        timers: &'static mut [&'static mut dyn Resource],
    ) -> Result<&'static mut Self, RuntimeError> {
        let inner = Self::get_inner();
        if let Some(_) = inner {
            return Err(RuntimeError::MultipleInitializations);
        };
        device::init_heap(device::heap_bottom(), heap_size);
        logging::init().expect("log initialization failed");
        inner.replace(Self {
            executor: executor::Executor::new(),
            resources: Resources::new(sys, input_pins, output_pins, pwm, channels, serials, timers),
        });
        let rt = Self::get();
        Ok(rt)
    }
    #[inline]
    fn get_inner() -> &'static mut Option<Runtime> {
        static mut RUNTIME: Option<Runtime> = None;
        unsafe { &mut RUNTIME }
    }
    #[inline]
    pub fn get() -> &'static mut Runtime {
        Self::get_inner().as_mut().expect("uninitialized runtime")
    }
    #[inline]
    pub(crate) fn get_resources() -> &'static mut Resources {
        &mut Self::get_inner()
            .as_mut()
            .expect("uninitialized runtime")
            .resources
    }
    pub fn get_resource(&'static mut self, uri: &str) -> Result<ResourceID, ResourceError> {
        self.resources.get_resource(uri)
    }
    pub fn run(&'static mut self) -> ! {
        loop {
            self.executor.run();
            crate::device::sleep();
        }
    }
    pub fn spawn_task(&'static mut self, task: Task) {
        self.executor.spawn(task);
    }
    pub(crate) fn register_waker(&'static mut self, trigger: &Event, waker: &Waker) {
        self.executor.register_waker(trigger, waker)
    }
}
