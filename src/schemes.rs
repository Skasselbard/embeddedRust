use core::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd, Hash)]
pub enum Scheme {
    H(Hardware),
    V(Virtual),
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd, Hash)]
pub enum Hardware {
    Memory,
    Bus,
    Analog,
    Digital,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd, Hash)]
pub enum Virtual {
    Event,
    Sys,
    Percent,
}
pub struct ParseSchemeError;

#[allow(unused)]
impl Scheme {
    #[inline]
    fn is_hardware(&self) -> bool {
        if let Scheme::H(_) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_virtual(&self) -> bool {
        if let Scheme::V(_) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_memory(&self) -> bool {
        if let Scheme::H(Hardware::Memory) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_bus(&self) -> bool {
        if let Scheme::H(Hardware::Bus) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_analog(&self) -> bool {
        if let Scheme::H(Hardware::Analog) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_digital(&self) -> bool {
        if let Scheme::H(Hardware::Digital) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_event(&self) -> bool {
        if let Scheme::V(Virtual::Event) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_sys(&self) -> bool {
        if let Scheme::V(Virtual::Sys) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_percent(&self) -> bool {
        if let Scheme::V(Virtual::Percent) = self {
            true
        } else {
            false
        }
    }
}

impl FromStr for Scheme {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "memory" => Ok(Scheme::H(Hardware::Memory)),
            "bus" => Ok(Scheme::H(Hardware::Bus)),
            "analog" => Ok(Scheme::H(Hardware::Analog)),
            "digital" => Ok(Scheme::H(Hardware::Digital)),
            "event" => Ok(Scheme::V(Virtual::Event)),
            "sys" => Ok(Scheme::V(Virtual::Sys)),
            "percent" => Ok(Scheme::V(Virtual::Percent)),
            _ => Err(ParseSchemeError),
        }
    }
}
