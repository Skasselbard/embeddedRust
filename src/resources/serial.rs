use super::{path::RawPath, Resource, ResourceMode};
use crate::device::{SerialID, SerialQueue, SerialQueueItem};
use crate::queues::{Consumer, Producer, Queue};
use crate::{io, schemes::Scheme};
use alloc::{boxed::Box, collections::VecDeque};
use core::{
    iter::Product,
    marker::PhantomData,
    task::{Context, Poll},
};
use io::SeekFrom;

/// Null
// const NUL: u8 = 0;
/// End Of Transmission
// const EOT: u8 = 4;

pub enum SerialState<'qu, HalWriter, HalReader, WordType> {
    PreAlloc {
        id: SerialID,
        writer: HalWriter,
        reader: HalReader,
    },
    PostAlloc(Serial<'qu, HalWriter, WordType>),
}
pub struct Serial<'qu, HalWriter, WordType> {
    id: SerialID,
    writer: HalWriter,
    read_buffer: Option<&'qu mut dyn Consumer<SerialQueueItem>>,
    queue: SerialQueue,
    _phantom_word: PhantomData<WordType>,
}

#[allow(unused)]
fn debug_word_type<WordType>(word: WordType, word_size: usize) {
    unsafe {
        if word_size == 1 {
            log::info!("{:?}", *(&word as *const WordType as *const u8));
        } else {
            let mut s = alloc::string::String::new();
            for i in 0..word_size {
                let byte = *(&word as *const WordType as *const u8).add(i);
                s.push(byte.into());
                s.push(',');
            }
            log::info!("[{}]", s);
        }
    }
}
impl<'q, HalWriter, WordType, WriteError> Resource for Serial<'q, HalWriter, WordType>
where
    HalWriter: embedded_hal::serial::Write<WordType, Error = WriteError>,
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
            // let read_length = buf.len() % Self::WORD_SIZE;
            // for i in 0..read_length {
            //     let mut word = match self.writer.read() {
            //         Ok(w) => w,
            //         Err(nb::Error::WouldBlock) => {
            //             return Poll::Pending;
            //         }
            //         Err(nb::Error::Other(_)) => {
            //             return Poll::Ready(Err(io::Error::Other));
            //         }
            //     };
            //     unsafe {
            //         // copy data into buffer
            //         core::intrinsics::write_bytes::<WordType>(
            //             buf[i * Self::WORD_SIZE] as *mut WordType,
            //             *(&mut word as *mut WordType as *mut u8),
            //             Self::WORD_SIZE,
            //         );
            //     }
            // }
            // Poll::Ready(Ok(read_length))
            Poll::Ready(Ok(0))
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
            let write_length = buf.len() / Self::WORD_SIZE;
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
                match self.writer.write(word) {
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
            match self.writer.flush() {
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

impl<'qu, HalWriter, HalReader, WordType, WriteError> Resource
    for SerialState<'qu, HalWriter, HalReader, WordType>
where
    HalWriter: embedded_hal::serial::Write<WordType, Error = WriteError>,
{
    fn poll_read(
        &mut self,
        context: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self {
            SerialState::PreAlloc { .. } => Poll::Ready(Err(io::Error::AddrNotAvailable)),
            SerialState::PostAlloc(s) => s.poll_read(context, scheme, mode, buf),
        }
    }
    fn poll_write<'a>(
        &'a mut self,
        context: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &'a [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self {
            SerialState::PreAlloc { .. } => Poll::Ready(Err(io::Error::AddrNotAvailable)),
            SerialState::PostAlloc(s) => s.poll_write(context, scheme, mode, buf),
        }
    }
    fn poll_flush(
        &mut self,
        context: &mut Context<'_>,
        scheme: Scheme,
        mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        match self {
            SerialState::PreAlloc { .. } => Poll::Ready(Err(io::Error::AddrNotAvailable)),
            SerialState::PostAlloc(s) => s.poll_flush(context, scheme, mode),
        }
    }
    fn poll_close(
        &mut self,
        context: &mut Context<'_>,
        scheme: Scheme,
        mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        match self {
            SerialState::PreAlloc { .. } => Poll::Ready(Err(io::Error::AddrNotAvailable)),
            SerialState::PostAlloc(s) => self.poll_close(context, scheme, mode),
        }
    }
    fn poll_seek(
        &mut self,
        cx: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        pos: SeekFrom,
    ) -> Poll<Result<u64, io::Error>> {
        match self {
            SerialState::PreAlloc { .. } => Poll::Ready(Err(io::Error::AddrNotAvailable)),
            SerialState::PostAlloc(s) => self.poll_seek(cx, scheme, mode, pos),
        }
    }
    fn path(&self) -> RawPath {
        match self {
            SerialState::PreAlloc { id, .. } => RawPath::Serial(*id),
            SerialState::PostAlloc(s) => RawPath::Serial(s.id),
        }
    }
    fn handle_event(&mut self) {
        match self {
            SerialState::PreAlloc { .. } => {}
            SerialState::PostAlloc(s) => s.handle_event(),
        }
    }
}

impl<'qu, HalWriter, WordType> Serial<'qu, HalWriter, WordType> {
    const WORD_SIZE: usize = core::mem::size_of::<WordType>();
}

impl<'qu, HalWriter, HalReader, WordType> SerialState<'qu, HalWriter, HalReader, WordType> {
    /// - Can be used befor the allocator is initialized
    /// - No allocations made here
    /// - Use init() before use
    pub fn new<ReadError, WriteError>(
        serial_id: SerialID,
        hal_writer: HalWriter,
        hal_reader: HalReader,
    ) -> Self
    where
        HalReader: embedded_hal::serial::Read<WordType, Error = ReadError> + 'static,
        HalWriter: embedded_hal::serial::Write<WordType, Error = WriteError>,
    {
        SerialState::PreAlloc {
            id: serial_id,
            writer: hal_writer,
            reader: hal_reader,
        }
    }
    /// allocates needed buffers on the heap
    pub fn init<ReadError, WriteError>(self) -> Self
    where
        HalReader: embedded_hal::serial::Read<WordType, Error = ReadError> + 'static,
        HalWriter: embedded_hal::serial::Write<WordType, Error = WriteError>,
    {
        match self {
            SerialState::PreAlloc { id, writer, reader } => {
                let queue = SerialQueue::new();
                self = SerialState::PostAlloc(Serial {
                    id,
                    writer,
                    queue,
                    read_buffer: None,
                    _phantom_word: PhantomData,
                });
                if let SerialState::PostAlloc(s) = self {
                    let (p, c) = unsafe { s.queue.split() };
                    s.read_buffer = Some(&mut c as &mut dyn Consumer<SerialQueueItem>);
                    crate::device::register_serial(id, Box::new(p));
                } else {
                    unreachable!()
                }
            }
            SerialState::PostAlloc(_) => {}
        }
        self
    }
}
