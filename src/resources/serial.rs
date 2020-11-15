use super::{path::RawPath, Resource, ResourceMode};
use crate::{device::SerialID, io, schemes::Scheme};
use core::{
    marker::PhantomData,
    task::{Context, Poll},
};
use io::SeekFrom;

/// Null
// const NUL: u8 = 0;
/// End Of Transmission
// const EOT: u8 = 4;

pub struct Serial<HalSerial, WordType> {
    id: SerialID,
    resource: HalSerial,
    _phantom_word: PhantomData<WordType>,
}

impl<HalSerial, WordType, ReadError, WriteError> Resource for Serial<HalSerial, WordType>
where
    HalSerial: embedded_hal::serial::Read<WordType, Error = ReadError>
        + embedded_hal::serial::Write<WordType, Error = WriteError>,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _scheme: Scheme,
        mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = mode {
            let read_length = buf.len() % Self::WORD_SIZE;
            for i in 0..read_length {
                let mut word = match self.resource.read() {
                    Ok(w) => w,
                    Err(nb::Error::WouldBlock) => {
                        return Poll::Pending;
                    }
                    Err(nb::Error::Other(_)) => {
                        return Poll::Ready(Err(io::Error::Other));
                    }
                };
                unsafe {
                    // copy data into buffer
                    core::intrinsics::write_bytes::<WordType>(
                        buf[i * Self::WORD_SIZE] as *mut WordType,
                        *(&mut word as *mut WordType as *mut u8),
                        Self::WORD_SIZE,
                    );
                }
            }
            Poll::Ready(Ok(read_length))
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_write<'a>(
        &'a mut self,
        _cx: &mut Context,
        _scheme: Scheme,
        mode: ResourceMode,
        buf: &'a [u8],
    ) -> Poll<Result<usize, io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = mode {
            let write_length = buf.len() % Self::WORD_SIZE;
            for i in 0..write_length {
                let mut word = unsafe { core::mem::zeroed::<WordType>() };
                unsafe {
                    // copy data from buffer
                    core::intrinsics::write_bytes::<WordType>(
                        &mut word as *mut WordType,
                        *(&buf[i * Self::WORD_SIZE] as *const u8),
                        Self::WORD_SIZE,
                    );
                };
                match self.resource.write(word) {
                    Ok(()) => {}
                    Err(nb::Error::WouldBlock) => {
                        return Poll::Pending;
                    }
                    Err(nb::Error::Other(_)) => {
                        return Poll::Ready(Err(io::Error::Other));
                    }
                };
            }
            Poll::Ready(Ok(write_length))
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_flush(
        &mut self,
        _: &mut Context<'_>,
        _scheme: Scheme,
        mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = mode {
            match self.resource.flush() {
                Ok(_) => Poll::Ready(Ok(())),
                Err(nb::Error::WouldBlock) => Poll::Pending,
                Err(nb::Error::Other(_)) => Poll::Ready(Err(io::Error::Other)),
            }
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_close(
        &mut self,
        _: &mut Context<'_>,
        _scheme: Scheme,
        _mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(
        &mut self,
        _cx: &mut Context,
        _scheme: Scheme,
        _mode: ResourceMode,
        _pos: SeekFrom,
    ) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn path(&self) -> RawPath {
        RawPath::Serial(self.id)
    }
    fn handle_event(&mut self) {}
}

impl<HalSerial, WordType, ReadError, WriteError> Serial<HalSerial, WordType>
where
    HalSerial: embedded_hal::serial::Read<WordType, Error = ReadError>
        + embedded_hal::serial::Write<WordType, Error = WriteError>,
{
    const WORD_SIZE: usize = core::mem::size_of::<WordType>();
    pub fn new(serial_id: SerialID, hal_serial: HalSerial) -> Self {
        Self {
            id: serial_id,
            resource: hal_serial,
            _phantom_word: PhantomData,
        }
    }
}
