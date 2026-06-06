#![cfg_attr(not(test), no_std)]

// Most of this code has been moved to the lib.rs file. This file was kept as a backup for personal reference.

#[cfg(test)]
extern crate std;

extern crate cortex_m;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free as critical_section;
use heapless::index_map::FnvIndexMap;
use serde::{Deserialize, Serialize};

// Enum of all configuration variable names
#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum ConfigVariable {
    heartbeat_ms,
    config_demo_variable1,
    // TODO: Add more configuration variables here
}

pub static CONFIG_U32_VARIABLES: Mutex<RefCell<FnvIndexMap<ConfigVariable, u32, 2>>> =
    Mutex::new(RefCell::new(FnvIndexMap::new()));

// Attempt to add configuration variables (temporary solution)

pub fn config_all_u32() {
    critical_section(|cs| {
        let mut config_u32 = CONFIG_U32_VARIABLES.borrow(cs).borrow_mut();
        config_u32.insert(ConfigVariable::heartbeat_ms, 500).unwrap();
        config_u32.insert(ConfigVariable::config_demo_variable1, 12345).unwrap();
    });
}

pub fn config_set_u32_variable(var_name: ConfigVariable, new_value: u32) -> Result <(), ()> {    
    // Potential for error handling here?
    
    critical_section(|cs| {
        if let Some(value) = CONFIG_U32_VARIABLES.borrow(cs).borrow_mut().get_mut(&var_name) {
            *value = new_value;
        }
    });

    Ok(())
}

pub fn config_get_u32_variable(var_name: ConfigVariable) -> Option<u32> {
    critical_section(|cs| {
        CONFIG_U32_VARIABLES.borrow(cs).borrow().get(&var_name).copied()
    })
}
