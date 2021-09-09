use crate::resources::serial::{InterruptConfig, InterruptConfigBuilder, InterruptHandler};
use crate::resources::ResourceError;
use core::{
    sync::atomic::{AtomicBool, Ordering},
    write,
};
use stm32f1xx_hal::device::{interrupt, USART1, USART2, USART3};
use stm32f1xx_hal::serial::{Rx, Tx};

pub type SerialWord = u8;
pub type SerialReadError = stm32f1xx_hal::serial::Error;
pub type SerialWriteError = core::convert::Infallible;

static mut USART1_BUFFER: Option<InterruptHandler> = None;
static mut USART2_BUFFER: Option<InterruptHandler> = None;
static mut USART3_BUFFER: Option<InterruptHandler> = None;

pub(crate) fn register_serial(queue: InterruptHandler) {
    unsafe {
        match queue.serial_id {
            SerialID::Usart1 => USART1_BUFFER = Some(queue),
            SerialID::Usart2 => USART2_BUFFER = Some(queue),
            SerialID::Usart3 => USART3_BUFFER = Some(queue),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Hash)]
pub enum SerialID {
    Usart1,
    Usart2,
    Usart3,
}
impl SerialID {
    #[inline]
    pub fn from_str(id: &str) -> Result<Self, ResourceError> {
        match id {
            "usart1" | "Usart1" | "USART1" => Ok(SerialID::Usart1),
            "usart2" | "Usart2" | "USART2" => Ok(SerialID::Usart2),
            "usart3" | "Usart3" | "USART3" => Ok(SerialID::Usart3),
            _ => Err(ResourceError::ParseError),
        }
    }
}

macro_rules! init_serial {
    ($serial:ty) => {{
        static CONFIG: DeviceConfig<$serial> = DeviceConfig {
            write_enabled: AtomicBool::new(false),
            reader: unsafe {
                &(core::mem::transmute::<(), Rx<$serial>>(())) as *const Rx<$serial>
                    as *mut Rx<$serial>
            },
            writer: unsafe {
                &(core::mem::transmute::<(), Tx<$serial>>(())) as *const Tx<$serial>
                    as *mut Tx<$serial>
            },
        };
        &CONFIG as &dyn InterruptConfig
    }};
}
pub(crate) struct SerialInterrupConfigBuilder;
impl InterruptConfigBuilder for SerialInterrupConfigBuilder {
    unsafe fn new(serial_id: SerialID) -> &'static dyn InterruptConfig {
        match serial_id {
            SerialID::Usart1 => init_serial!(USART1),
            SerialID::Usart2 => init_serial!(USART2),
            SerialID::Usart3 => init_serial!(USART3),
        }
    }
}

unsafe impl<T> Sync for DeviceConfig<T> {}
pub(crate) struct DeviceConfig<Serial> {
    write_enabled: AtomicBool,
    reader: *mut Rx<Serial>,
    writer: *mut Tx<Serial>,
}

macro_rules! serial_interrupts {
    ($serial:ty) => {
        impl InterruptConfig for DeviceConfig<$serial> {
            unsafe fn init(&mut self, _serial: SerialID) {
                self.write_enabled.store(false, Ordering::Relaxed);
                self.reader =
                    &mut (core::mem::transmute::<(), Rx<$serial>>(())) as *mut Rx<$serial>;
                self.writer =
                    &mut (core::mem::transmute::<(), Tx<$serial>>(())) as *mut Tx<$serial>;
                self.disable_write();
                self.disable_read();
            }
            #[inline]
            fn write_enabled(&self) -> bool {
                self.write_enabled.load(Ordering::Relaxed)
            }
            #[inline]
            unsafe fn disable_write(&self) {
                self.writer.as_mut().unwrap().unlisten();
                self.write_enabled.store(false, Ordering::Relaxed);
            }
            #[inline]
            unsafe fn enable_write(&self) {
                self.writer.as_mut().unwrap().listen();
                self.write_enabled.store(true, Ordering::Relaxed);
            }
            #[inline]
            unsafe fn disable_read(&self) {
                self.reader.as_mut().unwrap().unlisten();
            }
            #[inline]
            unsafe fn enable_read(&self) {
                self.reader.as_mut().unwrap().listen();
            }
        }
    };
}

serial_interrupts!(stm32f1xx_hal::pac::USART1);
serial_interrupts!(stm32f1xx_hal::pac::USART2);
serial_interrupts!(stm32f1xx_hal::pac::USART3);

#[interrupt]
fn USART1() {
    unsafe {
        USART1_BUFFER
            .as_mut()
            .expect("Usart1 not initialized")
            .handle()
    }
}
#[interrupt]
fn USART2() {
    unsafe {
        USART2_BUFFER
            .as_mut()
            .expect("Usart2 not initialized")
            .handle()
    }
}
#[interrupt]
fn USART3() {
    unsafe {
        USART3_BUFFER
            .as_mut()
            .expect("Usart3 not initialized")
            .handle()
    }
}
