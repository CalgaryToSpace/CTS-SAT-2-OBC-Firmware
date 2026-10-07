use crate::error::ConfigError;
use crate::shared;
use core::str::FromStr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU32, Ordering};
use serde_json_core::ser;

mod registry;
pub use registry::*;

pub static CONFIG_STORE: ConfigStore = ConfigStore {
    variables: CONFIG_VARIABLES,
};

pub struct ConfigStore {
    variables: &'static [&'static ConfigVariable],
}

pub struct ConfigVariable {
    pub name: &'static str,
    pub value: ConfigStorage,
}

impl ConfigVariable {
    pub fn to_json(&self, buffer: &mut [u8]) -> Result<usize, ser::Error> {
        #[derive(serde::Serialize)]
        struct Response<'a> {
            name: &'a str,
            value: ConfigValue,
        }

        serde_json_core::to_slice(
            &Response {
                name: self.name,
                value: self.value.get(),
            },
            buffer,
        )
    }
}

#[allow(dead_code)]
pub enum ConfigStorage {
    U32(AtomicU32),
    Bool(AtomicBool),
    F32(AtomicU32),
    I32(AtomicI32),
    U8(AtomicU8),
}

impl ConfigStorage {
    pub fn get(&self) -> ConfigValue {
        match self {
            Self::U32(value) => ConfigValue::U32(value.load(Ordering::Relaxed)),
            Self::Bool(value) => ConfigValue::Bool(value.load(Ordering::Relaxed)),
            Self::F32(value) => ConfigValue::F32(f32::from_bits(value.load(Ordering::Relaxed))),
            Self::I32(value) => ConfigValue::I32(value.load(Ordering::Relaxed)),
            Self::U8(value) => ConfigValue::U8(value.load(Ordering::Relaxed)),
        }
    }

    fn set(&self, value: ConfigValue) -> Result<(), ConfigError> {
        match (self, value) {
            (Self::U32(storage), ConfigValue::U32(value)) => {
                storage.store(value, Ordering::Relaxed)
            }
            (Self::Bool(storage), ConfigValue::Bool(value)) => {
                storage.store(value, Ordering::Relaxed)
            }
            (Self::F32(storage), ConfigValue::F32(value)) => {
                storage.store(value.to_bits(), Ordering::Relaxed)
            }
            (Self::I32(storage), ConfigValue::I32(value)) => {
                storage.store(value, Ordering::Relaxed)
            }
            (Self::U8(storage), ConfigValue::U8(value)) => storage.store(value, Ordering::Relaxed),
            _ => return Err(ConfigError::ConfigVariableNotThisType),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum ConfigValue {
    U32(u32),
    Bool(bool),
    F32(f32),
    I32(i32),
    U8(u8),
}

impl FromStr for ConfigValue {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (value_type, value_str) = shared::extract_function_and_args(s);

        macro_rules! parse_value {
            ($type:ty, $variant:path) => {
                value_str
                    .parse::<$type>()
                    .map($variant)
                    .map_err(|_| ConfigError::ConfigParseValueTypeError)
            };
        }

        match value_type {
            "u32" => parse_value!(u32, ConfigValue::U32),
            "bool" => parse_value!(bool, ConfigValue::Bool),
            "f32" => parse_value!(f32, ConfigValue::F32),
            "i32" => parse_value!(i32, ConfigValue::I32),
            "u8" => parse_value!(u8, ConfigValue::U8),
            _ => Err(ConfigError::ConfigVariableUnknownType),
        }
    }
}

impl ConfigStore {
    fn find(&self, name: &str) -> Result<&ConfigStorage, ConfigError> {
        self.variables
            .iter()
            .find(|variable| variable.name == name)
            .map(|variable| &variable.value)
            .ok_or(ConfigError::ConfigVariableNotFound)
    }

    pub fn get(&self, name: &str) -> Result<ConfigValue, ConfigError> {
        Ok(self.find(name)?.get())
    }

    pub fn set(&self, name: &str, value: ConfigValue) -> Result<(), ConfigError> {
        self.find(name)?.set(value)
    }

    pub fn get_all_vars(&self) -> &[&ConfigVariable] {
        self.variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_VAR1: ConfigVariable = ConfigVariable {
        name: "test_var1",
        value: ConfigStorage::U32(AtomicU32::new(0)),
    };
    static TEST_VAR2: ConfigVariable = ConfigVariable {
        name: "test_var2",
        value: ConfigStorage::Bool(AtomicBool::new(false)),
    };
    static STORE_TEST: ConfigStore = ConfigStore {
        variables: &[&TEST_VAR1, &TEST_VAR2],
    };

    #[test]
    fn test_config_get_all_vars() {
        let vars = STORE_TEST.get_all_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "test_var1");
        assert_eq!(vars[1].name, "test_var2");
    }
}
