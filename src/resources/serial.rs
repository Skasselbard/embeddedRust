use super::{path::RawPath, Resource, ResourceMode};
use crate::device::{SerialID, SerialReadError, SerialWord, SerialWriteError};
use crate::queues::{Consumer, Producer, Queue};
use crate::{io, schemes::Scheme};
use alloc::boxed::Box;
use bbqueue::{BBBuffer, ConstBBBuffer};
use core::task::{Context, Poll};
use io::SeekFrom;

type SerialQueue = BBBuffer<bbqueue::consts::U16>;
type SerialProducer = bbqueue::Producer<'static, bbqueue::consts::U16>;
type SerialConsumer = bbqueue::Consumer<'static, bbqueue::consts::U16>;

type Reader = dyn HalReader<Word = SerialWord, Error = SerialReadError>;
type Writer = dyn HalWriter<Word = SerialWord, Error = SerialWriteError>;
/// Null
// const NUL: u8 = 0;
/// End Of Transmission
// const EOT: u8 = 4;

pub trait HalReader {
    type Word;
    type Error;
    fn read(&mut self) -> Result<SerialWord, SerialReadError>;
    fn from_word(&self, word: Self::Word) -> &[u8];
}
pub trait HalWriter {
    type Word;
    type Error;
    const WORD_SIZE: usize = core::mem::size_of::<Self::Word>();

    fn write(&mut self, word: Self::Word) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
    // fn to_word(&self, bytes: [u8; Self::WORD_SIZE]) -> Self::Word;
}

impl<T> HalReader for T
where
    T: embedded_hal::serial::Read<SerialWord, Error = SerialReadError>,
{
    type Word = SerialWord;
    type Error = SerialReadError;

    fn read(&mut self) -> Result<SerialWord, SerialReadError> {
        match self.read() {
            Ok(w) => Ok(w),
            Err(nb::Error::WouldBlock) => unreachable!(), // TODO: is it unreachable?
            Err(nb::Error::Other(e)) => Err(e),
        }
    }

    fn from_word(&self, word: Self::Word) -> &[u8] {
        todo!()
    }
}

impl<T> HalWriter for T
where
    T: embedded_hal::serial::Write<SerialWord, Error = SerialWriteError>,
{
    type Word = SerialWord;
    type Error = SerialWriteError;

    fn write(&mut self, word: Self::Word) -> Result<(), Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

pub(crate) struct SerialBuffer {
    reader: &'static mut Reader,
    buffer: SerialProducer,
}

impl SerialBuffer {
    pub(crate) fn buffer_word(&mut self) {
        log::info!("U1");
        // if let Err(_) = self.buffer.enqueue(self.reader.read()) {
        //     panic!("serial buffer overflow");
        // }
        unimplemented!();
        // let e = Event::ExternalInterrupt(ExtiEvent::Gpio(Pin::new($channel, $port)));
    }
}

pub struct Serial<W, R> {
    id: SerialID,
    writer: W,
    reader: R,
    read_buffer: Option<SerialConsumer>,
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
impl<W, R> Resource for Serial<W, R>
where
    W: HalWriter<Word = SerialWord, Error = SerialWriteError>,
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
            // let write_length = buf.len() / Self::WORD_SIZE;
            // for i in 0..write_length {
            //     let mut word = unsafe { core::mem::zeroed::<WordType>() };
            //     unsafe {
            //         // copy data from buffer
            //         core::intrinsics::write_bytes::<WordType>(
            //             &mut word as *mut WordType,
            //             *(&buf[i * Self::WORD_SIZE] as *const u8),
            //             Self::WORD_SIZE,
            //         );
            //     };
            //     match self.writer.write(word) {
            //         Ok(()) => {}
            //         Err(nb::Error::WouldBlock) => {
            //             return Poll::Pending;
            //         }
            //         Err(nb::Error::Other(_)) => {
            //             return Poll::Ready(Err(io::Error::Other));
            //         }
            //     };
            // }
            // Poll::Ready(Ok(write_length))
            Poll::Ready(Ok(0))
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
            todo!();
            // match self.writer.flush() {
            //     Ok(_) => Poll::Ready(Ok(())),
            //     Err(nb::Error::WouldBlock) => Poll::Pending,
            //     Err(nb::Error::Other(_)) => Poll::Ready(Err(io::Error::Other)),
            // }
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

impl<W, R: 'static> Serial<W, R>
where
    W: HalWriter<Word = SerialWord, Error = SerialWriteError>,
    R: HalReader<Word = SerialWord, Error = SerialReadError>,
{
    pub fn new(serial_id: SerialID, hal_writer: W, hal_reader: R) -> Self {
        Self {
            id: serial_id,
            writer: hal_writer,
            reader: hal_reader,
            read_buffer: None,
        }
    }
    pub fn init(&'static mut self) {
        static QUEUE: SerialQueue = BBBuffer(ConstBBBuffer::new());
        let (p, c) = QUEUE.try_split().unwrap();
        let serial_buffer = SerialBuffer {
            reader: &mut self.reader,
            buffer: p,
        };
        crate::device::register_serial(self.id, serial_buffer);
        self.read_buffer = Some(c);
    }
}
