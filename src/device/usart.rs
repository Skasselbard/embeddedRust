use super::{Resource, ResourceError, ResourceID};
use crossbeam_queue::ArrayQueue;

use embedded_hal::serial::{Read, Write};

pub struct Usart<Bus> {
    bus: Bus,
}
impl<BUS> Usart<BUS> {
    pub fn new(bus: BUS) -> Self {
        Self { bus }
    }
}
impl<Bus> Resource for Usart<Bus>
where
    Bus: Read<u8> + Write<u8>,
{
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
        let read = 0;
        //while self.bus.read();
        Ok(1)
    }
    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
        let mut written = 0;
        for elem in buf {
            self.bus.write(*elem);
            written += 1;
        }
        Ok(written)
    }
    fn seek(&mut self, pos: usize) -> nb::Result<(), ResourceError> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), ResourceError> {
        Ok(())
    }
}
