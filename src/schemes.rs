use core::str::FromStr;

use crate::resources::{
    path::{IndexedPath, ResourceMode},
    ResourceError,
};

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

pub struct Memory {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Bus {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Analog {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Digital {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Event {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Sys {
    index: IndexedPath,
    mode: ResourceMode,
}
pub struct Percent {
    index: IndexedPath,
    mode: ResourceMode,
}
impl Memory {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
}
impl Bus {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
}
impl Analog {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
}
impl Digital {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
    //TODO: implement
    pub async fn is_high(&self) -> Result<bool, ResourceError> {
        unimplemented!()
    }
    pub async fn is_low(&self) -> Result<bool, ResourceError> {
        unimplemented!()
    }
    pub async fn set_high(&self) -> Result<(), ResourceError> {
        unimplemented!()
    }
    pub async fn set_low(&self) -> Result<(), ResourceError> {
        unimplemented!()
    }
}
impl Event {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
}
impl Sys {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
}
impl Percent {
    pub(crate) fn new(index: IndexedPath, mode: ResourceMode) -> Self {
        Self { index, mode }
    }
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
