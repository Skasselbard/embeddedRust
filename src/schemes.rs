use core::str::FromStr;
// Hardware
pub struct Memory;
pub struct Bus;
pub struct Analog;
pub struct Digital;

// Virtual
pub struct Event;
pub struct Sys;
pub struct Percent;

pub trait Hardware {}
impl Hardware for Memory {}
impl Hardware for Bus {}
impl Hardware for Analog {}
impl Hardware for Digital {}

pub trait Virtual {}
impl Virtual for Event {}
impl Virtual for Sys {}
impl Virtual for Percent {}

pub struct ParseSchemeError;

impl FromStr for Memory {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "memory" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Bus {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "bus" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Analog {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "analog" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Digital {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "digital" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Event {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "event" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Sys {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "sys" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
impl FromStr for Percent {
    type Err = ParseSchemeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "percent" {
            Ok(Self)
        } else {
            Err(ParseSchemeError)
        }
    }
}
