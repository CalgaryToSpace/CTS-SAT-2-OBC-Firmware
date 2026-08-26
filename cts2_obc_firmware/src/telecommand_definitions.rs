pub enum ReadinessLevel {
    Operation,
    _RecoveryOrExpert,
    _FlightTest,
    _GroundUsage,
    _HighRiskUnsafe
}

pub struct TelecommandDefinition { 
    pub name: &'static str,
    pub _num_parameters: u8,
    pub _readiness: ReadinessLevel,
}

pub const TELECOMMAND_DEFINITIONS: &[TelecommandDefinition] = &[
    TelecommandDefinition {
        name: "hello_world",
        _num_parameters: 0,
        _readiness: ReadinessLevel::Operation,
    },
    TelecommandDefinition {
        name: "get_sys_uptime",
        _num_parameters: 0,
        _readiness: ReadinessLevel::Operation,
    },
    TelecommandDefinition {
        name: "demo_command_with_arguments",
        _num_parameters: 1,
        _readiness: ReadinessLevel::Operation,
    },
    TelecommandDefinition {
        name: "get_config",
        _num_parameters: 1,
        _readiness: ReadinessLevel::Operation,
    },
    TelecommandDefinition {
        name: "set_config",
        _num_parameters: 2,
        _readiness: ReadinessLevel::Operation,
    },
];

const fn str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn assert_unique() {
    let mut i = 0;
    while i < TELECOMMAND_DEFINITIONS.len() {
        let name_i = TELECOMMAND_DEFINITIONS[i].name;
        let mut j = i + 1;
        while j < TELECOMMAND_DEFINITIONS.len() {
            let name_j = TELECOMMAND_DEFINITIONS[j].name;
            if str_eq(name_i, name_j) {
                panic!("Duplicate telecommand name");
            }
            j += 1;
        }
        i += 1;
    }
}

// Insert all compile time checks here.
// This will run at compile time
const _: () = assert_unique();
