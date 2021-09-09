use super::{path::RawPath, Resource, ResourceConfig, ResourceMode};
use crate::io;
use crate::{
    device::{
        SerialID, SerialInterrupConfigBuilder, SerialReadError, SerialWord, SerialWriteError,
    },
    events::{self, Event},
    Runtime,
};
use bbqueue::{BBBuffer, ConstBBBuffer};
use core::{
    mem::{size_of, MaybeUninit},
    task::Poll,
};
use embedded_hal::serial::{Read, Write};
use io::SeekFrom;

const WORD_SIZE: usize = core::mem::size_of::<SerialWord>();

type SerialQueue = BBBuffer<bbqueue::consts::U12>;
type SerialProducer = bbqueue::Producer<'static, bbqueue::consts::U12>;
type SerialConsumer = bbqueue::Consumer<'static, bbqueue::consts::U12>;
type SerialQueueItem = Result<SerialWord, SerialReadError>;

fn from_word(word: SerialWord) -> [u8; WORD_SIZE] {
    unsafe { core::mem::transmute(word) }
}
unsafe fn to_word(bytes: &[u8]) -> &SerialWord {
    assert_eq!(bytes.len(), WORD_SIZE);
    let bytes_array = bytes.first().unwrap() as *const u8;
    core::mem::transmute(bytes_array)
}
#[inline]
fn enqueue(buffer: &mut SerialProducer, item: &SerialQueueItem) {
    // get bbBuffer grant
    let mut grant = buffer
        .grant_exact(size_of::<SerialQueueItem>())
        .expect("Serial buffer overflow");
    // extract the byte array
    let buffer = grant.buf().as_mut_ptr();
    // copy bytes of item to the grant array
    unsafe {
        core::ptr::copy_nonoverlapping(
            item as *const SerialQueueItem as *const u8,
            buffer,
            size_of::<SerialQueueItem>(),
        );
    }
    // make it available for consumption
    grant.commit(size_of::<SerialQueueItem>())
}
#[inline]
fn dequeue(buffer: &mut SerialConsumer) -> Option<SerialQueueItem> {
    match buffer.read() {
        Ok(grant) => {
            // TODO: does this check something in a sufficiantly optimized debug binary?
            assert!(grant.len() >= size_of::<Result<SerialWord, SerialReadError>>());
            let item = &grant.buf()[0];
            unsafe {
                // get memory on stack
                let mut result: Result<SerialWord, SerialReadError> =
                    MaybeUninit::uninit().assume_init();
                // copy to new memory
                core::ptr::copy_nonoverlapping(
                    item,
                    &mut result as *mut Result<SerialWord, SerialReadError> as *mut u8,
                    size_of::<Result<SerialWord, SerialReadError>>(),
                );
                grant.release(size_of::<Result<SerialWord, SerialReadError>>());
                Some(result)
            }
        }
        Err(bbqueue::Error::InsufficientSize) => None,
        Err(bbqueue::Error::GrantInProgress) => {
            unreachable!(); // TODO: Only true single threaded
        }
        Err(bbqueue::Error::AlreadySplit) => {
            unreachable!()
        }
    }
}

pub(crate) struct InterruptHandler {
    pub serial_id: SerialID,
    reader: &'static mut dyn embedded_hal::serial::Read<SerialWord, Error = SerialReadError>,
    buffer: SerialProducer,
    config: &'static dyn InterruptConfig,
}
impl InterruptHandler {
    #[inline]
    pub(crate) fn handle(&mut self) {
        // check if it was a write interrupt
        if self.config.write_enabled() {
            // disable write interrupt and send event
            // writing is handled by the resource implementation
            // resource will be waked by event and try writing again
            unsafe { self.config.disable_write() };
        } else {
            // read_pin
            let result = match self.reader.read() {
                Ok(r) => Ok(r),
                Err(error) => {
                    // no it was a read error
                    match error {
                        nb::Error::Other(e) => Err(e),
                        // need to do something if read and write interrupt got mixed up...
                        // in theroy they should cancel each other out
                        nb::Error::WouldBlock => return,
                    }
                }
            };
            // store value
            enqueue(&mut self.buffer, &result);
        }
        events::push(Event::SerialEvent(self.serial_id));
    }
}
pub(crate) trait InterruptConfig {
    unsafe fn init(&mut self, serial: SerialID);
    fn write_enabled(&self) -> bool;
    unsafe fn disable_write(&self);
    unsafe fn enable_write(&self);
    unsafe fn disable_read(&self);
    unsafe fn enable_read(&self);
}
pub(crate) trait InterruptConfigBuilder {
    unsafe fn new(serial_id: SerialID) -> &'static dyn InterruptConfig;
}

pub struct Serial<W, R> {
    id: SerialID,
    writer: W,
    reader: R,
    read_buffer: Option<SerialConsumer>,
    interrupt_config: &'static dyn InterruptConfig,
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum SerialDirection {
    Read,
    Write,
}

impl<W, R> Resource for Serial<W, R>
where
    W: Write<SerialWord, Error = SerialWriteError>,
    R: Read<SerialWord, Error = SerialReadError>,
{
    fn poll_read(
        &mut self,
        config: &ResourceConfig,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = config.mode {
            unsafe { self.interrupt_config.enable_read() };
            let read_length = buf.len() / WORD_SIZE;
            let mut read_count = 0;
            for chunk in buf.chunks_exact_mut(size_of::<SerialWord>()) {
                match dequeue(self.read_buffer.as_mut().expect("uninitialized serial consumer")) {
                    Some(Ok(word)) => {
                        chunk.copy_from_slice(&from_word(word));
                        read_count += 1;
                    }
                    Some(Err(e)) => {
                        log::error!("{:?}: {:?}", self.id, e);
                        unsafe{self.interrupt_config.disable_read()};
                        return Poll::Ready(Err(io::Error::Interrupted))}
                        , // TODO: Log the Error
                    None => {
                        if read_count == 0{
                            Runtime::get().register_waker(
                                &Event::SerialEvent(self.id),
                                config.context.waker(),
                            );
                        return Poll::Pending;
                        }else{
                            return Poll::Ready(Ok(read_count))
                        }
                    }
                }
            }
            Poll::Ready(Ok(read_length))
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_write(
        &mut self,
        config: &ResourceConfig,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = config.mode {
            // only accept data that are a multiple of the word size
            if buf.len() % WORD_SIZE != 0 {
                return Poll::Ready(Err(io::Error::InvalidData));
            }
        } else {
            return Poll::Ready(Err(io::Error::InvalidInput));
        }
        let mut write_count = 0;
        for chunk in buf.chunks_exact(WORD_SIZE) {
            let word = unsafe { to_word(chunk) };
            match self.writer.write(*word) {
                Ok(()) => write_count += 1,
                Err(nb::Error::Other(e)) => {
                    log::error!("{:?}: {:?}", self.id, e);
                    return Poll::Ready(Err(io::Error::Other));
                }
                Err(nb::Error::WouldBlock) => {
                    if write_count == 0 {
                        // if we didn't read anything yet the job is pending
                        unsafe { self.interrupt_config.enable_write() };
                        Runtime::get()
                            .register_waker(&Event::SerialEvent(self.id), config.context.waker());
                        return Poll::Pending;
                    } else {
                        // otherwise we return the amount of bytes we managed to read so far
                        return Poll::Ready(Ok(write_count));
                    }
                }
            };
        }
        Poll::Ready(Ok(buf.len() / WORD_SIZE))
    }
    fn poll_flush(&mut self, config: &ResourceConfig) -> Poll<Result<(), io::Error>> {
        //TODO: use scheme
        if let ResourceMode::Default = config.mode {
            //FIXME: handle interrupts and polls correctly
            match self.writer.flush() {
                Ok(()) => Poll::Ready(Ok(())),
                Err(nb::Error::Other(_)) => Poll::Ready(Err(io::Error::Other)),
                Err(nb::Error::WouldBlock) => Poll::Pending,
            }
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_close(&mut self, _config: &ResourceConfig) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_seek(
        &mut self,
        _config: &ResourceConfig,
        _pos: SeekFrom,
    ) -> Poll<Result<u64, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn path(&self) -> RawPath {
        RawPath::Serial(self.id)
    }
    fn handle_event(&mut self) {}
}

impl<W, R> Serial<W, R>
where
    W: Write<SerialWord, Error = SerialWriteError>,
    R: Read<SerialWord, Error = SerialReadError>,
{
    pub fn new(serial_id: SerialID, hal_writer: W, hal_reader: R) -> Self {
        Self {
            id: serial_id,
            writer: hal_writer,
            reader: hal_reader,
            read_buffer: None,
            interrupt_config: unsafe { SerialInterrupConfigBuilder::new(serial_id) },
        }
    }
    pub fn init(&'static mut self) {
        static QUEUE: SerialQueue = BBBuffer(ConstBBBuffer::new());
        let (p, c) = QUEUE.try_split().unwrap();
        let interrupt_handler = InterruptHandler {
            serial_id: self.id,
            reader: &mut self.reader,
            buffer: p,
            config: self.interrupt_config,
        };
        crate::device::register_serial(interrupt_handler);
        self.read_buffer = Some(c);
    }
}
