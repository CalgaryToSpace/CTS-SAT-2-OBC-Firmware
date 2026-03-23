use crate::umbilical_uart::send_umbilical_uart;
pub mod demo_commands;

use crate::telecommand_implementation::demo_commands::{
    DEMO_ARGS, run_demo_command_with_arguments, run_hello_world_telecommand,
};
use crate::timekeeping::uptime_ms;

use cts2_obc_logic::scheduler::TaskArgs;

pub fn telecommand_hello_world(_args: TaskArgs) {
    run_hello_world_telecommand().ok();
}

pub fn telecommand_demo_command_with_arguments(_args: TaskArgs) {
    use cortex_m::interrupt::free as critical_section;
    if let Some(args) = critical_section(|cs| DEMO_ARGS.borrow(cs).borrow_mut().take()) {
        run_demo_command_with_arguments(args).ok();
    }
}

pub fn telecommand_get_sys_uptime(_args: TaskArgs) {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
}
