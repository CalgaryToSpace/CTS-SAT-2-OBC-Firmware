use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cts2_obc_telecommands::DemoCommandWithArgumentsArgs;
use rtt_target::rprintln;

use crate::umbilical_uart::send_umbilical_uart;
use cts2_obc_logic::scheduler::TaskArgs;

pub static DEMO_ARGS: Mutex<RefCell<Option<DemoCommandWithArgumentsArgs>>> =
    Mutex::new(RefCell::new(None));

pub fn run_hello_world_telecommand() -> Result<(), ()> {
    send_umbilical_uart(b"HELLO WORLD\r\n");

    Ok(())
}

pub fn run_demo_command_with_arguments(args: DemoCommandWithArgumentsArgs) -> Result<(), ()> {
    rprintln!(
        "DemoCommandWithArgumentsArgs: arg_u32={}, arg_u64={}, arg_bool={}, arg_f32={}, arg_f64={}, arg_nullable_u32={:?}\r\n",
        args.arg_u32,
        args.arg_u64,
        args.arg_bool,
        args.arg_f32,
        args.arg_f64,
        args.arg_nullable_u32
    );
    send_umbilical_uart(b"DEMO COMMAND WITH ARGUMENTS EXECUTED. See RTT output for details.\r\n");

    Ok(())
}

pub fn task_hello_world(_args: TaskArgs) {
    run_hello_world_telecommand().ok();
}

pub fn task_demo_command_with_arguments(_args: TaskArgs) {
    use cortex_m::interrupt::free as critical_section;
    if let Some(args) = critical_section(|cs| DEMO_ARGS.borrow(cs).borrow_mut().take()) {
        run_demo_command_with_arguments(args).ok();
    }
}
