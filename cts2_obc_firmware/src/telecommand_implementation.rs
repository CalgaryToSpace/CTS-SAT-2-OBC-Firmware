pub mod demo_commands;
pub mod config_commands;

use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
use crate::telecommand_implementation::demo_commands::{
    DEMO_ARGS, run_demo_command_with_arguments, run_hello_world_telecommand,
};
use crate::telecommand_implementation::config_commands::{get_config_variable_command, set_config_variable_command};
use cts2_obc_logic::scheduler::TaskArgs;
use rtt_target::rprintln;

pub fn telecommand_hello_world(_args: TaskArgs) {
    let _ = run_hello_world_telecommand();
}

pub fn telecommand_demo_command_with_arguments(_args: TaskArgs) {
    use cortex_m::interrupt::free as critical_section;
    if let Some(args) = critical_section(|cs| DEMO_ARGS.borrow(cs).borrow_mut().take()) {
        let _ = run_demo_command_with_arguments(args);
    }
}

pub fn telecommand_get_sys_uptime(_args: TaskArgs) {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
}

pub fn telecommand_get_config_variable(args: TaskArgs) {
    match args {
        TaskArgs::GetConfig(name) => {
            let _ = get_config_variable_command(name);
        }
        _ => {
            rprintln!("Error: Invalid arguments for telecommand_get_config_variable");
        }
    }
}

pub fn telecommand_set_config_variable(args: TaskArgs) {
    match args {
        TaskArgs::SetConfig(name, value) => {
            let _ = set_config_variable_command(name, value);
        }
        _ => {
            rprintln!("Error: Invalid arguments for telecommand_set_config_variable");
        }
    }
}
