use core::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd, Hash)]
pub enum Scheme {
    Memory,
    Bus,
    Analog,
    Digital,
    Event,
    Sys, //TODO: May should be a file scheme
    Percent,
}
pub enum SchemeError {
    ParseError,
}

#[allow(unused)]
impl Scheme {
    #[inline]
    fn is_memory(&self) -> bool {
        if let Scheme::Memory = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_bus(&self) -> bool {
        if let Scheme::Bus = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_analog(&self) -> bool {
        if let Scheme::Analog = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_digital(&self) -> bool {
        if let Scheme::Digital = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_event(&self) -> bool {
        if let Scheme::Event = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_sys(&self) -> bool {
        if let Scheme::Sys = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_percent(&self) -> bool {
        if let Scheme::Percent = self {
            true
        } else {
            false
        }
    }
}

impl FromStr for Scheme {
    type Err = SchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "memory" => Ok(Scheme::Memory),
            "bus" => Ok(Scheme::Bus),
            "analog" => Ok(Scheme::Analog),
            "digital" => Ok(Scheme::Digital),
            "event" => Ok(Scheme::Event),
            "sys" => Ok(Scheme::Sys),
            "percent" => Ok(Scheme::Percent),
            _ => Err(SchemeError::ParseError),
        }
    }
}
