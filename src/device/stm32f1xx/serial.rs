use crate::queues::single_threaded::ShortFixedSPSCQueue;
use crate::resources::serial::SerialBuffer;
use crate::resources::ResourceError;
use stm32f1xx_hal::device::interrupt;

pub type SerialWord = u8;
pub type SerialReadError = stm32f1xx_hal::serial::Error;
pub type SerialWriteError = core::convert::Infallible;
pub type SerialQueue =
    ShortFixedSPSCQueue<Result<SerialWord, SerialReadError>, heapless::consts::U16>;

static mut USART1_BUFFER: Option<SerialBuffer> = None;
static mut USART2_BUFFER: Option<SerialBuffer> = None;
static mut USART3_BUFFER: Option<SerialBuffer> = None;

pub(crate) fn register_serial(serial_id: SerialID, buffer: SerialBuffer) {
    unsafe {
        match serial_id {
            SerialID::Usart1 => USART1_BUFFER = Some(buffer),
            SerialID::Usart2 => USART2_BUFFER = Some(buffer),
            SerialID::Usart3 => USART3_BUFFER = Some(buffer),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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

#[interrupt]
fn USART1() {
    unsafe {
        USART1_BUFFER
            .as_mut()
            .expect("Usart1 not initialized")
            .buffer_word()
    }
}
#[interrupt]
fn USART2() {
    unsafe {
        USART2_BUFFER
            .as_mut()
            .expect("Usart2 not initialized")
            .buffer_word()
    }
}
#[interrupt]
fn USART3() {
    unsafe {
        USART3_BUFFER
            .as_mut()
            .expect("Usart3 not initialized")
            .buffer_word()
    }
}
