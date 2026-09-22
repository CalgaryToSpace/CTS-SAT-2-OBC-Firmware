use super::{ConfigStorage, ConfigVariable};
use core::sync::atomic::AtomicU32;

pub static HEARTBEAT_MS: ConfigVariable = ConfigVariable {
    name: "heartbeat_ms",
    value: ConfigStorage::U32(AtomicU32::new(1000)),
};

pub static CONFIG_DEMO_VARIABLE1: ConfigVariable = ConfigVariable {
    name: "config_demo_variable1",
    value: ConfigStorage::U32(AtomicU32::new(123)),
};

pub static CONFIG_VARIABLES: &[&ConfigVariable] = &[
    &HEARTBEAT_MS,
    &CONFIG_DEMO_VARIABLE1,
];
