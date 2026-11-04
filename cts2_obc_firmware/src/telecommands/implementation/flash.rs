use core::fmt::Write;

use crate::error::ExecuteCommandErr;
use crate::flash::wrappers;
use crate::umbilical_uart::send_umbilical_uart;
use cts2_obc_telecommands::validators::{MAX_FLASH_READ_BYTES, parse_flash_read};

pub fn flash_read(args: &str) -> Result<(), ExecuteCommandErr> {
    let (start, length) = parse_flash_read(args)?;
    let length = length as usize;
    let mut buffer = [0; MAX_FLASH_READ_BYTES as usize];
    wrappers::read(start, &mut buffer[..length])?;

    let mut response = heapless::String::<{ MAX_FLASH_READ_BYTES as usize * 2 }>::new();
    write!(response, "flash read from {} to {}: ", start, start + length as u32 - 1)?;
    for byte in &buffer[..length] {
        write!(response, "{byte:02X}")?;
    }

    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}
