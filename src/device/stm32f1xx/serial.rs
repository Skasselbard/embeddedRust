use crate::queues::single_threaded::ShortAllocFixedSPSCQueue;
use crate::queues::Producer;
use crate::resources::ResourceError;
use alloc::{boxed::Box, collections::VecDeque};
use core::marker::PhantomData;
use heapless::consts::U16;
use stm32f1xx_hal::serial::Serial;
use stm32f1xx_hal::{device::interrupt, prelude::_stm32_hal_time_U32Ext};

pub type SerialQueueItem = Result<u8, stm32f1xx_hal::serial::Error>;
pub type SerialQueue = ShortAllocFixedSPSCQueue<SerialQueueItem, U16>;

static mut USART1_BUFFER: Option<Box<dyn Producer<SerialQueueItem>>> = None;
// static mut USART1_BUFFER: Option<Box<dyn Producer<Result<u8, stm32f1xx_hal::serial::Error>>>> = None;

fn buffer_word<HalReader: 'static, WordType, ReadError>(
    reader: &mut HalReader,
    producer: &mut dyn Producer<SerialQueueItem>,
) where
    HalReader: embedded_hal::serial::Read<u8, Error = stm32f1xx_hal::serial::Error>,
{
    let word = match reader.read() {
        Ok(w) => Ok(w),
        Err(nb::Error::WouldBlock) => unreachable!(),
        Err(nb::Error::Other(e)) => Err(e),
    };
    if let Err(_) = producer.enqueue(word) {
        panic!("serial buffer overflow");
    }
    unimplemented!();
    // let e = Event::ExternalInterrupt(ExtiEvent::Gpio(Pin::new($channel, $port)));
    // log::info!("U1")
}

pub fn register_serial(serial_id: SerialID, p: Box<dyn Producer<SerialQueueItem>>) {
    unsafe {
        match serial_id {
            SerialID::Usart1 => USART1_BUFFER = Some(p),
            SerialID::Usart2 => {
                unimplemented!()
            }
            SerialID::Usart3 => {
                unimplemented!()
            }
            SerialID::Usart4 => {
                unimplemented!()
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum SerialID {
    Usart1,
    Usart2,
    Usart3,
    Usart4,
}
impl SerialID {
    #[inline]
    pub fn from_str(id: &str) -> Result<Self, ResourceError> {
        match id {
            "usart1" | "Usart1" | "USART1" => Ok(SerialID::Usart1),
            "usart2" | "Usart2" | "USART2" => Ok(SerialID::Usart2),
            "usart3" | "Usart3" | "USART3" => Ok(SerialID::Usart3),
            "usart4" | "Usart4" | "USART4" => Ok(SerialID::Usart4),
            _ => Err(ResourceError::ParseError),
        }
    }
}

macro_rules! check_interrupt {
    ($channel:ty) => {
        let e = Event::ExternalInterrupt(ExtiEvent::Gpio(Pin::new($channel, $port)));
        cortex_m::interrupt::free(|cs| {
            events::push(e, cs);
            pin.clear_interrupt_pending_bit();
        });
    };
}

#[interrupt]
fn USART1() {
    unsafe {
        buffer_word(
            reader,
            USART1_BUFFER.as_mut().expect("Usart1 not initialized"),
        )
    }
}
#[interrupt]
fn USART2() {
    panic!("irq");
}
#[interrupt]
fn USART3() {
    panic!("irq");
}
