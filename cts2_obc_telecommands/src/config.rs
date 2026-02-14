#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

extern crate cortex_m;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free as critical_section;
use heapless::index_map::FnvIndexMap;

#[derive(Debug, Eq, PartialEq, Hash)]
// Enum of all configuration variable names
#[allow(non_camel_case_types)]
pub enum ConfigVariable {
    heartbeat_ms,
    config_demo_variable1,
    // TODO: Add more configuration variables here
}

static CONFIG_U32_VARIABLES: Mutex<RefCell<FnvIndexMap<ConfigVariable, u32, 2>>> =
    Mutex::new(RefCell::new(FnvIndexMap::new()));

pub fn config_set_u32_variable(var_name: ConfigVariable, new_value: u32) {
    critical_section(|cs| {
        if let Some(value) = CONFIG_U32_VARIABLES.borrow(cs).borrow_mut().get_mut(&var_name) {
            *value = new_value;
        }
    });
}

pub fn config_get_u32_variable(var_name: ConfigVariable) -> Option<u32> {
    critical_section(|cs| {
        CONFIG_U32_VARIABLES.borrow(cs).borrow().get(&var_name).copied()
    })
}
