use super::{path::RawPath, Resource, ResourceError, ResourceMode};
use crate::{device::ExtiEvent, events::Event, schemes::Scheme, utilities::ByteWriter, Runtime};
use crate::{
    device::{Channel, Port},
    io,
};
use core::fmt::Write;
use core::{cmp::Ordering, task::Context, task::Poll};
use embedded_hal::digital::v2;
use io::SeekFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pin {
    channel: Channel,
    port: Port,
}
impl Pin {
    #[inline]
    pub fn from_str(pin: &str) -> Result<Self, ResourceError> {
        let channel = match pin {
            pin if pin.starts_with("pa") => Channel::A,
            pin if pin.starts_with("pb") => Channel::B,
            pin if pin.starts_with("pc") => Channel::C,
            pin if pin.starts_with("pd") => Channel::D,
            pin if pin.starts_with("pe") => Channel::E,
            _ => return Err(ResourceError::ParseError),
        };
        let port = match pin {
            pin if pin.ends_with("15") => Port::P15,
            pin if pin.ends_with("14") => Port::P14,
            pin if pin.ends_with("13") => Port::P13,
            pin if pin.ends_with("12") => Port::P12,
            pin if pin.ends_with("11") => Port::P11,
            pin if pin.ends_with("10") => Port::P10,
            pin if pin.ends_with("9") => Port::P09,
            pin if pin.ends_with("8") => Port::P08,
            pin if pin.ends_with("7") => Port::P07,
            pin if pin.ends_with("6") => Port::P06,
            pin if pin.ends_with("5") => Port::P05,
            pin if pin.ends_with("4") => Port::P04,
            pin if pin.ends_with("3") => Port::P03,
            pin if pin.ends_with("2") => Port::P02,
            pin if pin.ends_with("1") => Port::P01,
            pin if pin.ends_with("0") => Port::P00,
            _ => return Err(ResourceError::ParseError),
        };
        Ok(Self { channel, port })
    }
    #[inline]
    pub fn new(channel: Channel, port: Port) -> Self {
        Self { channel, port }
    }
    #[inline]
    pub fn channel(&self) -> Channel {
        self.channel
    }
    #[inline]
    pub fn port(&self) -> Port {
        self.port
    }
}
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum TriggerEdge {
    Rising,
    Falling,
    All,
}

pub struct InputPin<HalPin: 'static> {
    id: Pin,
    resource: HalPin,
    /// The amount of unhandles events
    events: u8,
}
pub struct OutputPin<HalPin: 'static> {
    id: Pin,
    resource: HalPin,
}

impl<HalPin, Error> Resource for InputPin<HalPin>
where
    HalPin: v2::InputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn poll_read(
        &mut self,
        context: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        // check if the mode kind is correct
        if let ResourceMode::Default = mode {
            match scheme {
                // read the pin normaly
                Scheme::Digital => match self.resource.is_high() {
                    Ok(res) => {
                        buf[0] = res as u8;
                        Poll::Ready(Ok(1))
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        Poll::Ready(Err(io::Error::Other))
                    }
                },
                Scheme::Percent => match self.resource.is_high() {
                    Ok(res) => {
                        let res = {
                            if res {
                                1.0f32
                            } else {
                                0.0f32
                            }
                        };
                        let mut buffer = ByteWriter::new(buf);
                        write!(buffer, "{}", res).map_err(|_| io::Error::InvalidInput)?;
                        Poll::Ready(Ok(buffer.written()))
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        Poll::Ready(Err(io::Error::Other))
                    }
                },
                // handle a gpio ecent
                Scheme::Event => {
                    if self.events > 0 {
                        self.events -= 1;
                        buf[0] = 0 as u8;
                        Poll::Ready(Ok(1))
                    } else {
                        Runtime::get().register_waker(
                            &Event::ExternalInterrupt(ExtiEvent::Gpio(self.id)),
                            context.waker(),
                        );
                        Poll::Pending
                    }
                }
                _ => Poll::Ready(Err(io::Error::InvalidInput)),
            }
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_write(
        &mut self,
        _cx: &mut Context,
        _scheme: Scheme,
        _mode: ResourceMode,
        _buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_flush(
        &mut self,
        _: &mut Context<'_>,
        _scheme: Scheme,
        _mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
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
        RawPath::Gpio(self.id)
    }
    fn handle_event(&mut self) {
        self.events += 1;
    }
}
impl<HalPin, Error> InputPin<HalPin>
where
    HalPin: v2::InputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin, hal_pin: HalPin) -> Self {
        InputPin {
            id: pin,
            resource: hal_pin,
            events: 0,
        }
    }
    pub fn get_pin(&self) -> Pin {
        self.id
    }
}

impl<HalPin, Error> Resource for OutputPin<HalPin>
where
    HalPin: v2::OutputPin<Error = Error> + Sync,
    Error: core::fmt::Display,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _scheme: Scheme,
        _mode: ResourceMode,
        _buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::AddrNotAvailable))
    }
    fn poll_write(
        &mut self,
        _cx: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        // check if the mode kind is correct
        if let ResourceMode::Default = mode {
            match scheme {
                Scheme::Digital => {
                    for byte in buf {
                        let res = match *byte != 0 {
                            true => self.resource.set_high(),
                            false => self.resource.set_low(),
                        };
                        if let Err(e) = &res {
                            log::error!("{}", e);
                            return Poll::Ready(Err(io::Error::Other));
                        }
                    }
                    Poll::Ready(Ok(buf.len()))
                }
                Scheme::Percent => {
                    if buf.len() != core::mem::size_of::<f32>() {
                        return Poll::Ready(Err(io::Error::InvalidInput));
                    }
                    // parse float
                    let percentage = match from_target_endianess!(f32, buf) {
                        Ok(v) => v,
                        Err(_) => return Poll::Ready(Err(io::Error::InvalidData)),
                    };
                    // check float boundaries
                    if percentage > 1.0 || percentage < 0.0 {
                        return Poll::Ready(Err(io::Error::InvalidData));
                    }
                    let res = if percentage < 0.5 {
                        self.resource.set_low()
                    } else {
                        self.resource.set_high()
                    };
                    if let Err(e) = &res {
                        log::error!("{}", e);
                        return Poll::Ready(Err(io::Error::Other));
                    }
                    Poll::Ready(Ok(core::mem::size_of::<f32>()))
                }
                _ => Poll::Ready(Err(io::Error::InvalidInput)),
            }
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    fn poll_flush(
        &mut self,
        _: &mut Context<'_>,
        _scheme: Scheme,
        _mode: ResourceMode,
    ) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
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
        RawPath::Gpio(self.id)
    }
    fn handle_event(&mut self) {}
}
impl<HalPin, Error> OutputPin<HalPin>
where
    HalPin: v2::OutputPin<Error = Error>,
    Error: core::fmt::Display,
{
    pub fn new(pin: Pin, hal_pin: HalPin) -> Self {
        OutputPin {
            id: pin,
            resource: hal_pin,
        }
    }
    pub fn get_pin(&self) -> Pin {
        self.id
    }
}

impl Ord for Pin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.port
            .cmp(&other.port)
            .then(self.channel.cmp(&other.channel))
    }
}

impl PartialOrd for Pin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl core::fmt::Display for Pin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "p{}{}", self.channel, self.port)
    }
}
