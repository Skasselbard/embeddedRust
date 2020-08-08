pub mod pwm;
pub mod usart;

#[cfg(feature = "stm32f1xx")]
mod stm32f1xx;
#[cfg(feature = "stm32f1xx")]
use stm32f1xx as dev;

use core::cmp::Ordering;

pub type ComponentConfiguration = dev::ComponentConfiguration;
pub type ExtiEvent = dev::ExtiEvent;
pub type PinMode = dev::PinMode;
pub type Direction = dev::Direction;
pub type Gpio = dev::Gpio;
pub type Channel = dev::Channel;
pub type Port = dev::Port;

/// Pin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pin {
    channel: Channel,
    port: Port,
}
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum TriggerEdge {
    Rising,
    Falling,
    All,
}

pub fn heap_bottom() -> usize {
    dev::heap_bottom()
}

pub fn sleep() {
    dev::sleep()
}

impl Pin {
    pub fn new(channel: Channel, port: Port) -> Self {
        Self { channel, port }
    }
    pub fn channel(&self) -> Channel {
        self.channel
    }
    pub fn port(&self) -> Port {
        self.port
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
