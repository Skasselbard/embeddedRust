use crate::{alloc::string::ToString, resources::Resource};
use crate::{
    io::{self, SeekFrom},
    schemes::Scheme,
};
use core::task::{Context, Poll};

use super::{path::RawPath, ResourceError, ResourceMode};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum SysPaths {
    Heap,
    SysClock,
}
impl SysPaths {
    pub fn from_str(path: &str) -> Result<Self, ResourceError> {
        match path {
            "heap" => Ok(SysPaths::Heap),
            "clock" | "sysclock" => Ok(SysPaths::SysClock),
            _ => Err(ResourceError::ParseError),
        }
    }
}
pub enum SysResource {
    Heap { size: usize },
    SysClock { clock: usize },
}

// TODO: handle schemes and modes
impl Resource for SysResource {
    fn poll_read(
        &mut self,
        _context: &mut Context,
        _scheme: Scheme,
        _mode: ResourceMode,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        let parsed = match self {
            SysResource::Heap { size } => size.to_string(),
            SysResource::SysClock { clock } => clock.to_string(),
        };
        let parsed = parsed.as_bytes();
        if buf.len() < parsed.len() {
            Poll::Ready(Err(io::Error::InvalidInput))
        } else {
            for i in 0..parsed.len() {
                buf[i] = parsed[i]
            }
            Poll::Ready(Ok(parsed.len()))
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
        RawPath::Sys(match self {
            SysResource::Heap { .. } => SysPaths::Heap,
            SysResource::SysClock { .. } => SysPaths::SysClock,
        })
    }
    fn handle_event(&mut self) {}
}
impl SysResource {
    pub fn new_heap(size: usize) -> Self {
        Self::Heap { size }
    }
    pub fn new_sysclock(clock_in_hertz: usize) -> Self {
        Self::SysClock {
            clock: clock_in_hertz,
        }
    }
}
