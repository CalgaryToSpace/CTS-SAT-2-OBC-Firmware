#![no_std]
use core::fmt::{Error, Write};

pub struct CustomCharBuffer {
    pub size: u8,
    pub char_buf: [u8; 128],
}

impl CustomCharBuffer {
    pub fn new() -> Self {
        CustomCharBuffer {
            size: 0,
            char_buf: [0; 128],
        }
    }
}

impl Write for CustomCharBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match self.write_char(c) {
                Ok(()) => {}
                Err(Error) => return Err(Error),
            }
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.char_buf[self.size as usize] = c as u8;
        self.size += 1;
        if self.size >= self.char_buf.len() as u8 {
            return Err(Error);
        }
        Ok(())
    }
}
