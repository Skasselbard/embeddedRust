use crate::Runtime;

use super::{gpio::Pin, pwm::PWMMode, sys::SysPaths, ResourceError};
use crate::device::SerialID;

#[derive(Copy, Clone, Eq, Debug, Hash)]
pub enum RawPath {
    Sys(SysPaths),
    Gpio(Pin),
    PWM(Pin, PWMMode),
    ADCPin(()),
    Serial(SerialID),
    Timer(()),
    Generic(()),
}

impl PartialEq for RawPath {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RawPath::Sys(path) => {
                if let RawPath::Sys(o_path) = other {
                    if path == o_path {
                        return true;
                    }
                }
            }
            RawPath::Gpio(pin) => {
                if let RawPath::Gpio(o_pin) = other {
                    if pin == o_pin {
                        return true;
                    }
                }
            }
            RawPath::PWM(pin, _) => {
                if let RawPath::PWM(o_pin, _) = other {
                    if pin == o_pin {
                        return true;
                    }
                }
            }
            RawPath::ADCPin(pin) => {
                if let RawPath::ADCPin(o_pin) = other {
                    if pin == o_pin {
                        return true;
                    }
                }
            }
            RawPath::Serial(id) => {
                if let RawPath::Serial(o_id) = other {
                    if id == o_id {
                        return true;
                    }
                }
            }
            RawPath::Timer(tim) => {
                if let RawPath::Timer(o_tim) = other {
                    if tim == o_tim {
                        return true;
                    }
                }
            }
            RawPath::Generic(index) => {
                if let RawPath::Generic(o_index) = other {
                    if index == o_index {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) enum IndexedPath {
    Sys(u8),
    InputGpio(u8),
    OutputGpio(u8),
    PWM(u8),
    ADCPin(u8),
    Serial(u8),
    Timer(u8),
    // Generic(u8),
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ResourceMode {
    Default,
    PWM(PWMMode),
}

impl RawPath {
    pub(crate) fn from_str(path: &str) -> Result<Self, ResourceError> {
        let mut segments = path.split('/');
        // TODO: are the errors useful?
        match segments.next() {
            Some("gpio") => Ok(RawPath::Gpio(Pin::from_str({
                segments.next().ok_or(ResourceError::ConversionError)?
            })?)),
            Some("pwm") => Ok(RawPath::PWM(
                Pin::from_str(segments.next().ok_or(ResourceError::ConversionError)?)?,
                PWMMode::from_str(segments.next().unwrap_or(""))?,
            )),
            Some("sys") => Ok(RawPath::Sys(SysPaths::from_str(
                segments.next().ok_or(ResourceError::ConversionError)?,
            )?)),
            Some("serial") => Ok(RawPath::Serial(SerialID::from_str(
                segments.next().ok_or(ResourceError::ConversionError)?,
            )?)),
            _ => Err(ResourceError::NotFound),
        }
    }
    pub(crate) fn resolve(self) -> Result<(IndexedPath, ResourceMode), ResourceError> {
        let resources = Runtime::get_resources();
        match self {
            RawPath::Sys(_sys_path) => Ok((
                IndexedPath::Sys(resources.search_resource_array(&self, resources.sys)?),
                ResourceMode::Default,
            )),
            RawPath::Gpio(_pin) => {
                if let Ok(index) = resources.search_resource_array(&self, resources.input_pins) {
                    Ok(((IndexedPath::InputGpio(index)), ResourceMode::Default))
                } else {
                    Ok((
                        (IndexedPath::OutputGpio(
                            resources.search_resource_array(&self, resources.output_pins)?,
                        )),
                        ResourceMode::Default,
                    ))
                }
            }
            RawPath::PWM(_pin, mode) => Ok((
                IndexedPath::PWM(resources.search_resource_array(&self, resources.pwm)?),
                ResourceMode::PWM(mode),
            )),
            RawPath::ADCPin(()) => Ok((
                IndexedPath::ADCPin(resources.search_resource_array(&self, resources.channels)?),
                ResourceMode::Default,
            )),
            RawPath::Serial(id) => Ok((
                IndexedPath::Serial(resources.search_resource_array(&self, resources.serials)?),
                ResourceMode::Default,
            )),
            RawPath::Timer(()) => Ok((
                IndexedPath::Timer(resources.search_resource_array(&self, resources.timers)?),
                ResourceMode::Default,
            )),
            RawPath::Generic(()) => {
                // Scheme::Event => self
                //     .search_virtual_resources(&parsed_uri)
                //     .map(|id| IndexedPath::Generic(id))
                //     .or({
                //         let id = NEXT_VIRTUAL_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                //         let event = Box::new(GpioEvent::from_uri(uri)?);
                //         self.generic_resources.insert(id, event);
                //         Ok(IndexedPath::Generic(id))
                //     }),
                // _ => unimplemented!(),
                unimplemented!()
            }
        }
    }
}
