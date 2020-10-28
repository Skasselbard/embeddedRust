use super::{path::RawPath, Resource, ResourceMode};
use crate::{io, schemes::Scheme};
use core::{
    marker::PhantomData,
    task::{Context, Poll},
};
use io::SeekFrom;

/// Null
// const NUL: u8 = 0;
/// End Of Transmission
// const EOT: u8 = 4;

pub struct Serial<HalSerial, Word> {
    //id: Pin,
    resource: HalSerial,
    _phantom_word: PhantomData<Word>,
}

impl<HalSerial, Word, Error> Resource for Serial<HalSerial, Word>
where
    HalSerial: embedded_hal::serial::Read<Word, Error = Error>
        + embedded_hal::serial::Write<Word, Error = Error>,
    Error: core::fmt::Display,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _scheme: Scheme,
        mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        if let ResourceMode::Default = mode {
            let read_length = buf.len() % Self::WORD_SIZE;
            for i in 0..read_length {
                let mut word = match self.resource.read() {
                    Ok(w) => w,
                    Err(nb::Error::WouldBlock) => {
                        return Poll::Pending;
                    }
                    Err(nb::Error::Other(e)) => {
                        log::error!("Serial Error: {}", e);
                        return Poll::Ready(Err(io::Error::Other));
                    }
                };
                unsafe {
                    // copy data into buffer
                    core::intrinsics::write_bytes::<Word>(
                        buf[i * Self::WORD_SIZE] as *mut Word,
                        *(&mut word as *mut Word as *mut u8),
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
        if let ResourceMode::Default = mode {
            let write_length = buf.len() % Self::WORD_SIZE;
            for i in 0..write_length {
                let mut word = unsafe { core::mem::zeroed::<Word>() };
                unsafe {
                    // copy data from buffer
                    core::intrinsics::write_bytes::<Word>(
                        &mut word as *mut Word,
                        *(&buf[i * Self::WORD_SIZE] as *const u8),
                        Self::WORD_SIZE,
                    );
                };
                match self.resource.write(word) {
                    Ok(()) => {}
                    Err(nb::Error::WouldBlock) => {
                        return Poll::Pending;
                    }
                    Err(nb::Error::Other(e)) => {
                        log::error!("Serial Error: {}", e);
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
        if let ResourceMode::Default = mode {
            match self.resource.flush() {
                Ok(_) => Poll::Ready(Ok(())),
                Err(nb::Error::WouldBlock) => Poll::Pending,
                Err(nb::Error::Other(e)) => Poll::Ready(Err(io::Error::Other)),
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
        unimplemented!()
        // RawPath::Serial(self.id, PWMMode::Default)
    }
    fn handle_event(&mut self) {}
}
impl<HalSerial, Word> Serial<HalSerial, Word>
where
    HalSerial: embedded_hal::serial::Read<Word> + embedded_hal::serial::Write<Word>,
{
    const WORD_SIZE: usize = core::mem::size_of::<Word>();
    pub fn new(hal_serial: HalSerial) -> Self {
        Self {
            resource: hal_serial,
            _phantom_word: PhantomData,
        }
    }
}
