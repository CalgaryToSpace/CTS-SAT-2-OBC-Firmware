use crate::{error::ExecuteCommandErr, umbilical_uart::send_umbilical_uart};

pub fn run_hello_world_telecommand(_args: &str) -> Result<(), ExecuteCommandErr> {
    send_umbilical_uart(b"HELLO WORLD\r\n");

    Ok(())
}
