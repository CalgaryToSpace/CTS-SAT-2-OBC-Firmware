use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
use core::fmt::Write;
use cts2_obc_logic::CustomCharBuffer;
pub mod demo_commands;

pub fn get_sys_uptime_ms_telecommand() -> Result<(), ()> {
    let mut buff = CustomCharBuffer::new();
    let sys_time = uptime_ms();
    write!(&mut buff, "System Uptime: {} ms\r\n", sys_time).unwrap();
    send_umbilical_uart(&buff.char_buf);
    Ok(())
}
