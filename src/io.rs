pub trait Read {
    type Error;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

pub trait Write {
    type Error;
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}