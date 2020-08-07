// // use embedded_rust_devices::resources::{Resource, ResourceError};
// use log::error;

// use embedded_hal::serial::{Read, Write};

// const NUL: u8 = 0;
// /// end of text
// const ETX: u8 = 3;
// /// end of transmission
// const EOT: u8 = 4;

// pub struct Usart<Bus> {
//     read_index: usize,
//     write_index: usize,
//     bus: Bus,
// }
// impl<BUS> Usart<BUS> {
//     pub fn new(bus: BUS) -> Self {
//         Self {
//             bus,
//             read_index: 0,
//             write_index: 0,
//         }
//     }
// }
// // impl<Bus, BusReadError, BusWriteError> Resource for Usart<Bus>
// // where
// //     Bus: Read<u8, Error = BusReadError> + Write<u8, Error = BusWriteError> + Send,
// //     BusReadError: core::fmt::Debug,
// //     BusWriteError: core::fmt::Debug,
// // {
// //     fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
// //         loop {
// //             match self.bus.read() {
// //                 Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
// //                 Err(nb::Error::Other(e)) => {
// //                     self.read_index = 0; // reset on failure
// //                     error!("{:?}", e);
// //                     return Err(nb::Error::Other(ResourceError::BusError));
// //                 }
// //                 Ok(byte) => {
// //                     buf[self.read_index] = byte;
// //                     self.read_index += 1;
// //                     match byte {
// //                         NUL | EOT | ETX => return Ok(self.read_index),
// //                         _ => {}
// //                     }
// //                 }
// //             };
// //             if self.read_index == buf.len() {
// //                 return Ok(self.read_index);
// //             }
// //         }
// //     }
// //     fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
// //         for elem in buf {
// //             match self.bus.write(*elem) {
// //                 Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
// //                 Err(nb::Error::Other(e)) => {
// //                     self.write_index = 0; // reset on failure
// //                     error!("{:?}", e);
// //                     return Err(nb::Error::Other(ResourceError::BusError));
// //                 }
// //                 Ok(()) => {}
// //             }
// //             self.write_index += 1;
// //         }
// //         let written = self.write_index;
// //         self.write_index = 0;
// //         Ok(written)
// //     }
// //     fn seek(&mut self, _pos: usize) -> nb::Result<(), ResourceError> {
// //         Ok(())
// //     }
// //     fn flush(&mut self) -> nb::Result<(), ResourceError> {
// //         Ok(())
// //     }
// // }
