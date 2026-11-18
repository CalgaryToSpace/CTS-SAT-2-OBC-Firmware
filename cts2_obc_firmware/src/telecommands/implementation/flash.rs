use core::fmt::Write;

use crate::error::ExecuteCommandErr;
use crate::flash::wrappers;
use crate::umbilical_uart::{MAX_TELECOMMAND_STR_LENGTH, send_umbilical_uart};
use cts2_obc_telecommands::error::ParsedTelecommandErr;
use cts2_obc_telecommands::validators::{
    MAX_FLASH_READ_BYTES, parse_flash_read, parse_flash_write,
};

pub fn flash_read(args: &str) -> Result<(), ExecuteCommandErr> {
    let (start, length) = parse_flash_read(args)?;
    let length = length as usize;
    let mut buffer = [0; MAX_FLASH_READ_BYTES as usize];
    wrappers::read(start, &mut buffer[..length])?;

    let mut response = heapless::String::<{ MAX_FLASH_READ_BYTES as usize * 2 + 64 }>::new();
    write!(
        response,
        "flash read from {} to {}: ",
        start,
        start + length as u32 - 1
    )?;
    for byte in &buffer[..length] {
        write!(response, "{byte:02X}")?;
    }

    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}

pub fn flash_write(args: &str) -> Result<(), ExecuteCommandErr> {
    let (start, hex) = parse_flash_write(args)?;
    let mut buffer = [0; MAX_TELECOMMAND_STR_LENGTH / 2];
    let data = buffer
        .get_mut(..hex.len() / 2)
        .ok_or(ParsedTelecommandErr::InvalidWriteData)?;
    for (index, byte) in data.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| ParsedTelecommandErr::InvalidWriteData)?;
    }

    wrappers::write(start, data)?;

    let mut response = heapless::String::<64>::new();
    write!(response, "flash wrote {} bytes at {}", data.len(), start)?;
    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}
