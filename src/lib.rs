#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod executor;
mod logging;

#[macro_use]
pub mod device;

pub mod events;
pub mod io;
pub mod resources;
pub mod schemes;

pub use executor::Task;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::str::from_utf8_mut;
use core::sync::atomic::AtomicU8;
use core::task::Waker;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use device::GpioEvent;
use events::Event;
use nom_uri::Uri;
use resources::{Resource, ResourceID};
use schemes::{Hardware, Scheme, Virtual};

pub struct Runtime {
    sys: &'static mut [&'static mut dyn Resource],
    input_pins: &'static mut [&'static mut dyn Resource],
    output_pins: &'static mut [&'static mut dyn Resource],
    pwm: &'static mut [&'static mut dyn Resource],
    channels: &'static mut [&'static mut dyn Resource],
    serials: &'static mut [&'static mut dyn Resource],
    timers: &'static mut [&'static mut dyn Resource],
    virtual_resources: BTreeMap<u8, Box<dyn Resource>>,
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
            sys,
            input_pins,
            output_pins,
            pwm,
            channels,
            serials,
            timers,
            virtual_resources: BTreeMap::new(),
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
    fn get_resource_object(&'static mut self, id: &ResourceID) -> &mut dyn Resource {
        match id {
            ResourceID::Sys(index) => *self.sys.get_mut(*index as usize).unwrap(),
            ResourceID::InputGpio(index) => *self.input_pins.get_mut(*index as usize).unwrap(),
            ResourceID::OutputGpio(index) => *self.output_pins.get_mut(*index as usize).unwrap(),
            ResourceID::PWM(index) => *self.pwm.get_mut(*index as usize).unwrap(),
            ResourceID::Channel(index) => *self.channels.get_mut(*index as usize).unwrap(),
            ResourceID::Serial(index) => *self.serials.get_mut(*index as usize).unwrap(),
            ResourceID::Timer(index) => *self.timers.get_mut(*index as usize).unwrap(),
            ResourceID::Virtual(key) => self
                .virtual_resources
                .get_mut(key)
                .expect("Missing virtual Resource")
                .as_mut(),
        }
    }
    pub fn get_resource(&'static mut self, uri: &str) -> Result<ResourceID, RuntimeError> {
        use core::convert::TryFrom;
        use core::str::FromStr;
        static NEXT_VIRTUAL_ID: AtomicU8 = AtomicU8::new(0);
        let parsed_uri = Uri::try_from(uri).or(Err(RuntimeError::UriParseError))?;
        match Scheme::from_str(parsed_uri.scheme()).map_err(|_| RuntimeError::ResourceNotFound)? {
            Scheme::H(hw) => match hw {
                Hardware::Digital => match parsed_uri.path() {
                    path if path.starts_with("gpio") => {
                        if let Ok(int) = self.search_resource_array(&parsed_uri, self.input_pins) {
                            Ok(ResourceID::InputGpio(int))
                        } else {
                            self.search_resource_array(&parsed_uri, self.output_pins)
                                .map(|int| ResourceID::OutputGpio(int))
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            Scheme::V(v) => match v {
                Virtual::Event => self
                    .search_virtual_resources(&parsed_uri)
                    .map(|id| ResourceID::Virtual(id))
                    .or({
                        let id =
                            NEXT_VIRTUAL_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                        let event = Box::new(GpioEvent::from_uri(uri)?);
                        self.virtual_resources.insert(id, event);
                        Ok(ResourceID::Virtual(id))
                    }),
                _ => unimplemented!(),
            },
        }
    }
    fn search_resource_array(
        &self,
        search_uri: &Uri,
        array: &'static [&'static mut dyn Resource],
    ) -> Result<u8, RuntimeError> {
        let mut buf_array = [0u8; 255];
        let buffer: &mut str = from_utf8_mut(&mut buf_array).unwrap();
        for i in 0..array.len() {
            if &array[i].to_uri(buffer) == search_uri {
                return Ok(i as u8);
            }
        }
        Err(RuntimeError::ResourceNotFound)
    }
    fn search_virtual_resources(&self, search_uri: &Uri) -> Result<u8, RuntimeError> {
        let mut buf_array = [0u8; 255];
        let buffer: &mut str = from_utf8_mut(&mut buf_array).unwrap();
        self.virtual_resources
            .iter()
            .find_map(|(k, v)| {
                if &v.to_uri(buffer) == search_uri {
                    Some((k, v))
                } else {
                    None
                }
            })
            .ok_or(RuntimeError::ResourceNotFound)
            .map(|(k, _v)| *k)
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
