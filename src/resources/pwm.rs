use super::{gpio::Pin, path::RawPath, Resource, ResourceError, ResourceMode};
use crate::{io, schemes::Scheme, utilities::ByteWriter};
use core::{
    convert::{TryFrom, TryInto},
    fmt::{Debug, Write},
    ops::Mul,
    task::{Context, Poll},
};
use io::SeekFrom;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum PWMMode {
    Default,
    MaxDuty,
}
impl PWMMode {
    pub fn from_str(mode: &str) -> Result<Self, ResourceError> {
        match mode {
            "" => Ok(PWMMode::Default),
            "max" | "maxduty" => Ok(PWMMode::MaxDuty),
            _ => Err(ResourceError::ParseError),
        }
    }
}
pub struct PWMPin<HalPWMPin: 'static> {
    id: Pin,
    resource: HalPWMPin,
}

impl<HalPWMPin, Duty> Resource for PWMPin<HalPWMPin>
where
    HalPWMPin: embedded_hal::PwmPin<Duty = Duty>,
    Duty: Mul + TryFrom<usize> + Into<usize>,
    <Duty as TryFrom<usize>>::Error: Debug,
{
    fn poll_read(
        &mut self,
        _context: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        if let ResourceMode::PWM(mode) = mode {
            let duty: usize = self.resource.get_duty().into();
            let mut buffer = ByteWriter::new(buf);
            match mode {
                PWMMode::Default => match scheme {
                    Scheme::Analog => {
                        write!(buffer, "{}", duty).map_err(|_| io::Error::InvalidInput)?
                    }
                    Scheme::Percent => {
                        let max: usize = self.resource.get_max_duty().into();
                        write!(buffer, "{}", (duty as f32 / max as f32))
                            .map_err(|_| io::Error::InvalidInput)?;
                    }
                    _ => return Poll::Ready(Err(io::Error::InvalidInput)),
                },
                PWMMode::MaxDuty => {
                    let max: usize = self.resource.get_max_duty().into();
                    write!(buffer, "{}", max).map_err(|_| io::Error::InvalidInput)?;
                }
            }
            Poll::Ready(Ok(buffer.written()))
        } else {
            Poll::Ready(Err(io::Error::InvalidInput))
        }
    }
    /// takes a f32 percentage (between 0.0 and 1.0) and sets duty accordingly
    fn poll_write(
        &mut self,
        _cx: &mut Context,
        scheme: Scheme,
        mode: ResourceMode,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        if let ResourceMode::PWM(mode) = mode {
            match mode {
                PWMMode::Default => {
                    match scheme {
                        Scheme::Analog => {
                            if buf.len() != core::mem::size_of::<usize>() {
                                return Poll::Ready(Err(io::Error::InvalidInput));
                            }
                            // parse
                            let duty = match from_target_endianess!(usize, buf) {
                                Ok(v) => v,
                                Err(_) => return Poll::Ready(Err(io::Error::InvalidData)),
                            };
                            // disable if duty = 0
                            if duty == 0 {
                                self.resource.disable();
                                return Poll::Ready(Ok(core::mem::size_of::<usize>()));
                            }
                            if duty > self.resource.get_max_duty().into() {
                                Poll::Ready(Err(io::Error::InvalidData))
                            } else {
                                self.resource
                                    .set_duty(duty.try_into().expect("pwm duty conversion error"));
                                self.resource.enable();
                                Poll::Ready(Ok(core::mem::size_of::<usize>()))
                            }
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
                            // disable if duty = 0
                            if percentage == 0.0 {
                                self.resource.disable();
                                return Poll::Ready(Ok(core::mem::size_of::<f32>()));
                            }
                            // check float boundaries
                            if percentage > 1.0 || percentage < 0.0 {
                                return Poll::Ready(Err(io::Error::InvalidData));
                            }
                            // convert to target format
                            let max = self.resource.get_max_duty().into();
                            let duty = (max as f32 * percentage) as usize;
                            // set duty and enable pwm
                            self.resource
                                .set_duty(duty.try_into().expect("pwm duty conversion error"));
                            self.resource.enable();
                            Poll::Ready(Ok(core::mem::size_of::<f32>()))
                        }
                        _ => Poll::Ready(Err(io::Error::InvalidData)),
                    }
                }
                PWMMode::MaxDuty => Poll::Ready(Err(io::Error::InvalidInput)),
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
        RawPath::PWM(self.id, PWMMode::Default)
    }
    fn handle_event(&mut self) {}
}
impl<HalPWMPin, Duty> PWMPin<HalPWMPin>
where
    HalPWMPin: embedded_hal::PwmPin<Duty = Duty>,
    Duty: Mul + TryFrom<usize> + Into<usize>,
    <Duty as TryFrom<usize>>::Error: Debug,
{
    // const DUTY_SIZE: usize = core::mem::size_of::<Duty>();
    pub fn new(pin: Pin, mut hal_pin: HalPWMPin) -> Self {
        hal_pin.set_duty(0.try_into().unwrap());
        hal_pin.disable();
        PWMPin {
            id: pin,
            resource: hal_pin,
        }
    }
}
