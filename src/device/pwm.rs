use core::fmt::Write;
use embedded_rust_devices::resources::{ByteWriter, Resource, ResourceError};

use embedded_hal::PwmPin;

pub struct Pwm<Channel> {
    channel: Channel,
    enabled: bool,
}
impl<Channel, Duty> Pwm<Channel>
where
    Channel: PwmPin<Duty = Duty>,
    Duty: core::ops::Mul + From<f32> + Into<f32>,
{
    pub fn new(mut channel: Channel) -> Self {
        channel.set_duty(0.0.into());
        channel.disable();
        Self {
            channel,
            enabled: false,
        }
    }
    fn set_pwm(&mut self, percentage: f32) {
        let max: f32 = self.channel.get_max_duty().into();
        let duty: f32 = max * percentage;
        self.channel.set_duty(duty.into());
        match self.enabled {
            true => {
                if duty == 0.0 {
                    self.channel.disable();
                    self.enabled = false;
                }
            }
            false => {
                if duty != 0.0 {
                    self.channel.enable();
                    self.enabled = true;
                }
            }
        }
    }
}
// impl<Channel, Duty> Resource for Pwm<Channel>
// where
//     Channel: PwmPin<Duty = Duty>,
//     Duty: core::ops::Mul + From<f32> + Into<f32>,
// {
//     fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ResourceError> {
//         let duty: f32 = self.channel.get_duty().into();
//         let max: f32 = self.channel.get_max_duty().into();
//         let mut buffer = ByteWriter::new(buf);
//         write!(buffer, "{}", (duty / max));
//         Ok(buffer.written())
//     }
//     fn write(&mut self, buf: &[u8]) -> nb::Result<usize, ResourceError> {
//         let buf = core::str::from_utf8(buf).map_err(|e| nb::Error::Other(e.into()))?;
//         let percentage = buf.parse::<f32>().map_err(|e| nb::Error::Other(e.into()))?;
//         self.set_pwm(percentage);
//         Ok(0)
//     }
//     fn seek(&mut self, _pos: usize) -> nb::Result<(), ResourceError> {
//         Ok(())
//     }
//     fn flush(&mut self) -> nb::Result<(), ResourceError> {
//         Ok(())
//     }
// }
