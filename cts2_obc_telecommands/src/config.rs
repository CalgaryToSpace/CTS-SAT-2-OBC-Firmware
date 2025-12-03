/**
 * - Enum of all configuration variable names.
 * - Struct that we create a global static singleton that contains all variable names.
 * - Create a getter and a setter telecommand (getter arg: name of variable, 
 *   setter arg: name of variable, value)
 * - Add configuration variable: heartbeat in ms (for testing to start)
 * - Add configuration variable: config_demo_variable1
 */

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

#[derive(Debug)]


// Enum of all configuration variable names
enum ConfigVariable {
    CONFIG_int_demo_var1,
    CONFIG_int_demo_var2,
    TASK_heartbeat_ms,
    // TODO: Add more configuration variables here
}
