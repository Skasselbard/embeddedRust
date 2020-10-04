pub struct ByteWriter<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
}
pub struct StrWriter<'a> {
    byte_writer: ByteWriter<'a>,
}
impl<'a> ByteWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }
    pub fn buffer(self) -> &'a mut [u8] {
        let (o, _) = self.buffer.split_at_mut(self.cursor);
        o
    }
    pub fn written(&self) -> usize {
        self.cursor
    }
}
impl<'a> StrWriter<'a> {
    pub fn new(buffer: &'a mut str) -> Self {
        Self {
            byte_writer: ByteWriter::new(unsafe { buffer.as_bytes_mut() }),
        }
    }
    pub fn buffer(self) -> Result<&'a mut str, core::str::Utf8Error> {
        core::str::from_utf8_mut(self.byte_writer.buffer())
    }
    pub fn written(&self) -> usize {
        self.byte_writer.written()
    }
}
impl<'a> core::fmt::Write for ByteWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        if self.buffer.len() - self.cursor < s.len() {
            log::error!("fmt::Write: Buffer to small");
            Err(core::fmt::Error)
        } else {
            for i in 0..s.len() {
                self.buffer[self.cursor] = s[i];
                self.cursor += 1;
            }
            Ok(())
        }
    }
}
impl<'a> core::fmt::Write for StrWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.byte_writer.write_str(s)
    }
}
impl<'buf> From<&'buf mut [u8]> for ByteWriter<'buf> {
    fn from(buffer: &'buf mut [u8]) -> Self {
        Self::new(buffer)
    }
}
impl<'buf> From<&'buf mut str> for ByteWriter<'buf> {
    fn from(buffer: &'buf mut str) -> Self {
        Self::new(unsafe { buffer.as_bytes_mut() })
    }
}
impl<'buf> From<&'buf mut str> for StrWriter<'buf> {
    fn from(buffer: &'buf mut str) -> Self {
        Self::new(buffer)
    }
}
