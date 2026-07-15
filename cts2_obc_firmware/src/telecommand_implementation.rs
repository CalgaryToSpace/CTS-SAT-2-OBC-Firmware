use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
pub mod demo_commands;

pub fn get_sys_uptime_ms_telecommand() -> Result<(), ()> {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
    Ok(())
}

pub fn my_name_telecommand() -> Result<(), ()> {
    let buff = b"Hello, my name is Thomas\r\n";
    send_umbilical_uart(buff);
    Ok(())
}
