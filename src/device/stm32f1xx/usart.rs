use crate::device::DeviceInterrupt;
use crate::events::{Event, Priority};
use crate::RuntimeError;
use crossbeam_queue::{ArrayQueue, PushError};
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::device::{interrupt, USART1};
use stm32f1xx_hal::serial::{self, Rx, Tx};

pub const QUEUE_LENGTH: usize = 32;
// pub static mut USART1_OBJ: Option<Usart<USART1>> = None;
static mut USART1_SINGLTN: Option<(Rx<USART1>, ArrayQueue<Result<u8, serial::Error>>)> = None;

pub struct Usart<Bus> {
    tx: Tx<Bus>,
    buffer: &'static ArrayQueue<Result<u8, serial::Error>>,
}

impl<Bus> Read<u8> for Usart<Bus>
where
    Rx<Bus>: Read<u8, Error = serial::Error>,
{
    type Error = serial::Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.buffer.pop() {
            Err(_) => Err(nb::Error::WouldBlock),
            Ok(result) => result.map_err(|e| nb::Error::Other(e)),
        }
    }
}

impl<Bus> Write<u8> for Usart<Bus>
where
    Tx<Bus>: Write<u8, Error = core::convert::Infallible>,
{
    type Error = core::convert::Infallible;
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(byte)
    }
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl Usart<USART1> {
    pub fn new(tx: Tx<USART1>, rx: Rx<USART1>) -> Result<Self, RuntimeError> {
        unsafe {
            match USART1_SINGLTN {
                Some(_) => return Err(RuntimeError::MultipleInitializations),
                None => USART1_SINGLTN = Some((rx, ArrayQueue::new(QUEUE_LENGTH))),
            }
        };
        Ok(Self {
            tx,
            buffer: unsafe { &USART1_SINGLTN.as_ref().unwrap().1 }, // just initialized
        })
    }
}
#[interrupt]
fn USART1() {
    let usart = unsafe {
        USART1_SINGLTN
            .as_mut()
            .expect("acces to uninitialized usart1")
    };
    let byte = block!(usart.0.read()); // should not block because we are in rdy interrupt
    usart.1.push(byte).expect("usart1: buffer filled");
    crate::events::push(
        Event::ResourceEvent(DeviceInterrupt::USART1),
        Priority::Critical,
    )
    .expect("filled event queue");
}

/// usart with default values
/// Pins: PA9, PA10
#[macro_export]
macro_rules! usart1 {
    ( $gpioa:expr, $peripherals:expr, $rcc:expr, $afio:expr, $clocks:expr) => {{
        use crate::device::stm32f1xx::*;
        use stm32f1xx_hal::serial::{self, Event};
        let tx = $gpioa.pa9.into_alternate_push_pull(&mut $gpioa.crh);
        let rx = $gpioa.pa10;
        let mut serial = serial::Serial::usart1(
            $peripherals.USART1,
            (tx, rx),
            &mut $afio.mapr,
            stm32f1xx_hal::serial::Config::default(),
            $clocks,
            &mut $rcc.apb2,
        );
        serial.listen(Event::Rxne);
        let (tx, rx) = serial.split();
        let serial = Usart::new(tx, rx).unwrap();
        crate::device::usart::Usart::new(serial)
    }};
}
/// usart with default values
/// Pins: PA2, PA3
#[macro_export]
macro_rules! usart2 {
    ( $gpioa:expr, $peripherals:expr, $rcc:expr, $afio:expr, $clocks:expr) => {{
        let tx = $gpioa.pa2.into_alternate_push_pull(&mut $gpioa.crl);
        let rx = $gpioa.pa3;
        let serial = stm32f1xx_hal::serial::Serial::usart2(
            $peripherals.USART2,
            (tx, rx),
            &mut $afio.mapr,
            stm32f1xx_hal::serial::Config::default(),
            $clocks,
            &mut $rcc.apb1,
        );
        crate::device::usart::Usart::new(serial)
    }};
}
/// usart with default values
/// Pins: PB!=, PA11
#[macro_export]
macro_rules! usart3 {
    ( $gpiob:expr, $peripherals:expr, $rcc:expr, $afio:expr, $clocks:expr) => {{
        let tx = $gpiob.pb10.into_alternate_push_pull(&mut $gpiob.crh);
        let rx = $gpiob.pb11;
        let serial = stm32f1xx_hal::serial::Serial::usart3(
            $peripherals.USART3,
            (tx, rx),
            &mut $afio.mapr,
            stm32f1xx_hal::serial::Config::default(),
            $clocks,
            &mut $rcc.apb1,
        );
        crate::device::usart::Usart::new(serial)
    }};
}
