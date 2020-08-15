#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod executor;
mod logging;

#[macro_use]
pub mod device;

pub mod events;
pub mod resources;
pub mod schemes;

pub use executor::Task;

use alloc::boxed::Box;
use core::task::Waker;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use device::ComponentConfiguration;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use events::Event;
use heapless::consts::*;
use heapless::Vec;
use log::trace;
use nom_uri::Uri;
use resources::{Resource, ResourceID};
use schemes::{Hardware, Scheme, Virtual};

pub struct Runtime {
    sys: &'static [&'static mut dyn Resource],
    input_pins: &'static [&'static mut dyn Resource],
    output_pins: &'static [&'static mut dyn Resource],
    pwm: &'static [&'static mut dyn Resource],
    channels: &'static [&'static mut dyn Resource],
    serials: &'static [&'static mut dyn Resource],
    timers: &'static [&'static mut dyn Resource],
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
        sys: &'static [&'static mut dyn Resource],
        input_pins: &'static [&'static mut dyn Resource],
        output_pins: &'static [&'static mut dyn Resource],
        pwm: &'static [&'static mut dyn Resource],
        channels: &'static [&'static mut dyn Resource],
        serials: &'static [&'static mut dyn Resource],
        timers: &'static [&'static mut dyn Resource],
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
        });
        let rt = Self::get();
        // rt.configure(resource_configuration);
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
    // fn get_resource_object(&'static mut self, id: &ResourceID) -> &'static mut dyn Resource {
    //     match id {
    //         ResourceID::Sys(index) => {
    //             *self.sys.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::InputGpio(index) => {
    //             *self.input_pins.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::OutputGpio(index) => {
    //             *self.output_pins.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::PWM(index) => {
    //             *self.pwm.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::Channel(index) => {
    //             *self.channels.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::Serial(index) => {
    //             *self.serials.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //         ResourceID::Timer(index) => {
    //             *self.timers.get_mut(*index as usize).unwrap() as &'static mut dyn Resource
    //         }
    //     }
    // }
    pub fn get_resource(&mut self, uri: &str) -> Result<ResourceID, RuntimeError> {
        use core::convert::TryFrom;
        use core::str::FromStr;
        let parsed_uri = Uri::try_from(uri).or(Err(RuntimeError::UriParseError))?;
        let resource_array = match Scheme::from_str(parsed_uri.scheme())
            .map_err(|e| RuntimeError::ResourceNotFound)?
        {
            Scheme::H(Hardware::Digital) => match parsed_uri.path() {
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
        };
        Err(RuntimeError::ResourceNotFound)
    }
    fn search_resource_array(
        &self,
        search_uri: &Uri,
        array: &'static [&'static mut dyn Resource],
    ) -> Result<u8, RuntimeError> {
        use core::str::from_utf8_mut;
        let mut buf_array = [0u8; 255];
        let buffer: &mut str = from_utf8_mut(&mut buf_array).unwrap();
        for i in 0..array.len() {
            //TODO:
            // if array[i].to_uri(buffer) == search_uri {
            //     return Ok(i as u8);
            // }
        }
        unimplemented!()
    }
    // fn add_resource(&mut self, resource: &'static mut dyn Resource) -> ResourceID {
    //     let id = self.static_resources.len();
    //     match self.static_resources.push(resource) {
    //         Ok(()) => {}
    //         Err(_resource) => panic!("filled resource queue"),
    //     }
    //     ResourceID(id as u8)
    // }
    pub fn run(&mut self) -> ! {
        //TODO: enable interrupts here
        trace!("run");
        loop {
            self.executor.run();
            trace!("sleep");
            crate::device::sleep();
        }
    }
    pub fn spawn_task(&mut self, task: Task) {
        trace!("spawn");
        self.executor.spawn(task);
    }
    // fn configure(&mut self, configurations: &[ComponentConfiguration]) {
    //     for configuration in configurations {
    //         let resource: &dyn Resource = match configuration {
    //             ComponentConfiguration::Gpio(gpio) => gpio,
    //             _ => unimplemented!(),
    //         };
    //         // TODO: maybe cloning is somehow possible
    //         let resource = unsafe { &mut *(resource as *const dyn Resource as *mut dyn Resource) };
    //         self.add_resource(resource);
    //     }
    // }
    pub(crate) fn register_waker(&mut self, trigger: &Event, waker: &Waker) {
        self.executor.register_waker(trigger, waker)
    }
}
