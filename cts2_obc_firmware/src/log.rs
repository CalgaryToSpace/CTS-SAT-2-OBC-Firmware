use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::free as critical_section;
use cortex_m::interrupt::Mutex;
use heapless::String;

#[derive(Copy, Clone)]
#[repr(u32)]
enum LogSeverity {
    Debug = 1 << 0,
    Normal = 1 << 1,
    Warning = 1 << 2,
    Error = 1 << 3,
    Critical = 1 << 4,
}

impl LogSeverity {
    fn as_str(&self) -> &'static str {
        match self {
            LogSeverity::Debug => "DEBUG",
            LogSeverity::Normal => "NORMAL",
            LogSeverity::Warning => "WARNING",
            LogSeverity::Error => "ERROR",
            LogSeverity::Critical => "CRITICAL",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
enum LogSink {
    UhfRadio = 1 << 0,
    File = 1 << 1,
    UmbilicalUart = 1 << 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum LogSystem {
    Obc,
    UhfRadio,
    UmbilicalUart,
    Gnss,
    Mpi,
    Eps,
    Boom,
    Adcs,
    Lfs,
    Flash,
    AntennaDeploy,
    Log,
    Telecommand,
    UnitTest,
    Unknown,
}

impl LogSystem {
    fn as_str(&self) -> &'static str {
        match self {
            LogSystem::Obc => "OBC",
            LogSystem::UhfRadio => "UHF_RADIO",
            LogSystem::UmbilicalUart => "UMBILICAL_UART",
            LogSystem::Gnss => "GNSS",
            LogSystem::Mpi => "MPI",
            LogSystem::Eps => "EPS",
            LogSystem::Boom => "BOOM",
            LogSystem::Adcs => "ADCS",
            LogSystem::Lfs => "LFS",
            LogSystem::Flash => "FLASH",
            LogSystem::AntennaDeploy => "ANTENNA_DEPLOY",
            LogSystem::Log => "LOG",
            LogSystem::Telecommand => "TELECOMMAND",
            LogSystem::UnitTest => "UNIT_TEST",
            LogSystem::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Copy, Clone)]
struct LogSystemConfig{
    system: LogSystem,
    name: &'static str,
    log_file_path: &'static str,
    file_logging_enabled: bool,
    severity_mask: u32,
}

impl LogSystemConfig {
    fn format(&self) -> Option<String<LOG_FORMATTED_MESSAGE_MAX_LENGTH>> {
        let mut message = String::new();

        write!(
            message,
            "{}: file_logging={}, log_file='{}', severity_mask={:#07b}",
            self.name,
            self.file_logging_enabled,
            self.log_file_path,
            self.severity_mask
        ).ok()?;

        Some(message)
    }
}

#[derive(Copy, Clone)]
struct LogSinkConfig {
    sink: LogSink,
    name: &'static str,
    enabled: bool,
    severity_mask: u32,
}

impl LogSinkConfig {
    fn format(&self) -> Option<String<LOG_FORMATTED_MESSAGE_MAX_LENGTH>> {
        let mut message = String::new();

        write!(
            message,
            "{}: enabled={}, severity_mask={:#07b}",
            self.name,
            self.enabled,
            self.severity_mask
        ).ok()?;

        Some(message)
    }
}

#[derive(Copy, Clone)]
struct LogConfiguration {
    sinks: [LogSinkConfig; 3],
    systems: [LogSystemConfig; 15],
}

//sink configurations
static LOG_SINKS: Mutex<RefCell<[LogSinkConfig; 3]>> = Mutex::new(RefCell::new([
    LogSinkConfig {
        sink: LogSink::UhfRadio,
        name: "UHF_RADIO",
        enabled: false,
        severity_mask: LOG_SEVERITY_MASK_ALL_EXCEPT_DEBUG,
    },
    LogSinkConfig {
        sink: LogSink::File,
        name: "FILE",
        enabled: false,
        severity_mask: LOG_SEVERITY_MASK_ALL_EXCEPT_DEBUG,
    },
    LogSinkConfig {
        sink: LogSink::UmbilicalUart,
        name: "UMBILICAL",
        enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL_EXCEPT_DEBUG,
    },
]));

static LOG_SYSTEMS : Mutex<RefCell<[LogSystemConfig; 15]>> = Mutex::new(RefCell::new([
    LogSystemConfig {
        system: LogSystem::Obc,
        name: "OBC",
        log_file_path: "/logs/obc_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::UhfRadio,
        name: "RADIO",
        log_file_path: "/logs/uhf_radio_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::UmbilicalUart,
        name: "UART",
        log_file_path: "/logs/umbilical_uart_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Gnss,
        name: "GNSS",
        log_file_path: "/logs/gnss_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Mpi,
        name: "MPI",
        log_file_path: "/logs/mpi_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Eps,
        name: "EPS",
        log_file_path: "/logs/eps_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Boom,
        name: "BOOM",
        log_file_path: "/logs/boom_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Adcs,
        name: "ADCS",
        log_file_path: "/logs/adcs_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Lfs,
        name: "LFS",
        log_file_path: "/logs/lfs_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Flash,
        name: "FLASH",
        log_file_path: "/logs/flash_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::AntennaDeploy,
        name: "ANTS",
        log_file_path: "/logs/antenna_deploy_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Log,
        name: "LOG",
        log_file_path: "/logs/log_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Telecommand,
        name: "TCMD",
        log_file_path: "/logs/telecommand_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::UnitTest,
        name: "TEST",
        log_file_path: "/logs/unit_test_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    },
    LogSystemConfig {
        system: LogSystem::Unknown,
        name: "UNK",
        log_file_path: "/logs/unknown_system.log",
        file_logging_enabled: true,
        severity_mask: LOG_SEVERITY_MASK_ALL,
    }
]));

// Log severity masks for filtering log messages
const LOG_SEVERITY_MASK_ALL: u32 =
    (LogSeverity::Debug as u32) | (LogSeverity::Normal as u32) | (LogSeverity::Warning as u32) | (LogSeverity::Error as u32) | (LogSeverity::Critical as u32);

const LOG_SEVERITY_MASK_ALL_EXCEPT_DEBUG: u32 = LOG_SEVERITY_MASK_ALL & !(LogSeverity::Debug as u32);

const LOG_SINK_MASK_ALL: u32 =
    (LogSink::UhfRadio as u32) | (LogSink::File as u32) | (LogSink::UmbilicalUart as u32);

//checks whether a severity level is enabled by the severity mask
fn severity_allowed(mask: u32, severity: LogSeverity) -> bool {
    mask & (severity as u32) != 0
}

const LOG_FORMATTED_MESSAGE_MAX_LENGTH: usize = 185;

//formats a log message with its subsystem and severity data
fn format_log_message(system: &LogSystem,severity: &LogSeverity,message: &str,) -> Option<String<LOG_FORMATTED_MESSAGE_MAX_LENGTH>> {
    let mut formatted_message = String::new();

    if write!(formatted_message,"[{}:{}]: {}",
        system.as_str(),severity.as_str(),message
    ).is_err(){

        return None;

    }

    Some(formatted_message)
}

// Format and send a log message to the selected sinks
// Parameters:
//     system: subsystem generating the message
//     severity: message severity
//     sink_mask: bitmask of sinks to send the message to
//     message: message text
fn log_message(system: LogSystem, severity: LogSeverity, sink_mask: u32, message: &str) {

    //check if the system is enabled for the given severity level
    let system_config  = critical_section(|cs| {
        LOG_SYSTEMS.borrow(cs).borrow()
            .iter()
            .find(|system_config| system_config.system == system)
            .map(|system_config| (system_config.severity_mask, system_config.file_logging_enabled))
    });

    let Some((system_severity_mask, file_logging_enabled)) = system_config else {
        return;
    };

    if !severity_allowed(system_severity_mask, severity) {
        return;
    }

    // Format the message with its subsystem and severity
    let formatted_message = match format_log_message(&system, &severity, message) {
        Some(message) => message,
        None => return,
    };

    let sink_configs = critical_section(|cs| *LOG_SINKS.borrow(cs).borrow());

    for sink_config in sink_configs.iter() {
        if !sink_config.enabled {
            continue;
        }

        if sink_mask & (sink_config.sink as u32) == 0 {
            continue;
        }

        if !severity_allowed(sink_config.severity_mask, severity) {
            continue;
        }

        // Send to appropriate sink
        match sink_config.sink {
            LogSink::UhfRadio => {
                log_to_uhf_radio(&formatted_message);
            }
            LogSink::File => {
                if file_logging_enabled {
                    log_to_file(&formatted_message);
                }
            }
            LogSink::UmbilicalUart => {
                log_to_uart(&formatted_message);
            }
        }
    }
}

// Check logging sink status
// Parameters:
//     sink: the sink to check
// Returns:
//     false: sink is disabled
//     true: sink is enabled
fn log_is_sink_enabled(sink: LogSink) -> bool {
    critical_section(|cs| {
        let sink_configs = LOG_SINKS.borrow(cs).borrow();

        for sink_config in sink_configs.iter() {
            if sink_config.sink == sink {
                return sink_config.enabled;
            }
        }

        false
    })
}

// Enable or disable a logging sink
// Parameters:
//     sink: the sink to update
//     enabled: true: enable the sink; false: disable the sink
fn log_set_sink_enabled(sink: LogSink, enabled: bool) {
    let sink_name = critical_section(|cs| {
        let mut sink_configs = LOG_SINKS.borrow(cs).borrow_mut();

        for sink_config in sink_configs.iter_mut() {
            if sink_config.sink == sink {
                sink_config.enabled = enabled;
                return Some(sink_config.name);
            }
        }

        None
    });

    let name = match sink_name {
        Some(name) => name,
        None => return,
    };

    let mut message = String::<LOG_FORMATTED_MESSAGE_MAX_LENGTH>::new();
    let write_result = if enabled {
        write!(message, "Enabled the {} sink", name)
    } else {
        write!(message, "Disabled the {} sink", name)
    };

    if write_result.is_err() {
            return;
        }

    log_message(
            LogSystem::Log,
            LogSeverity::Normal,
            LOG_SINK_MASK_ALL,
            &message,
        );
}


// Enable or disable debugging messages for a sink
// Parameters:
//     sink: the sink to update
//     enabled: true: enable debugging; false: disable debugging
// Returns:
//     None
fn log_set_sink_debugging_enabled(sink: LogSink, enabled: bool) {
    let sink_name = critical_section(|cs| {
        let mut sink_configs = LOG_SINKS.borrow(cs).borrow_mut();

        for sink_config in sink_configs.iter_mut() {
            if sink_config.sink != sink {
                continue;
            }

            if enabled {
                sink_config.severity_mask |= LogSeverity::Debug as u32;
            } else {
                sink_config.severity_mask &= !(LogSeverity::Debug as u32);
            }

            return Some(sink_config.name);
        }

        None
    });

    if let Some(name) = sink_name {
        let mut message = String::<LOG_FORMATTED_MESSAGE_MAX_LENGTH>::new();
        let write_result = if enabled {
            write!(message, "Enabled debugging on the {} sink", name)
        } else {
            write!(message, "Disabled debugging on the {} sink", name)
        };

        if write_result.is_err() {
            return;
        }

        log_message(
            LogSystem::Log,
            LogSeverity::Normal,
            LOG_SINK_MASK_ALL,
            &message,
        );
    }
}


// Set the logging severity bitfield mask for a subsystem
// Parameters:
//     system: the system to update
//     severity_mask: the new severity mask
// Returns:
//     None
fn log_set_system_severity_mask(system: LogSystem, severity_mask: u32) {
    let system_name = critical_section(|cs| {
        let mut system_configs = LOG_SYSTEMS.borrow(cs).borrow_mut();

        for system_config in system_configs.iter_mut() {
            if system_config.system == system {
                system_config.severity_mask = severity_mask;
                return Some(system_config.name);
            }
        }

        None
    });

    if let Some(name) = system_name {
        let mut message = String::<LOG_FORMATTED_MESSAGE_MAX_LENGTH>::new();
        if write!(
            message,
            "Updated severity mask for {} to {:#07b}",
            name, severity_mask
        )
        .is_err()
        {
            return;
        }

        log_message(
            LogSystem::Log,
            LogSeverity::Normal,
            LOG_SINK_MASK_ALL,
            &message
        );
    };
}

// Enable or disable file logging for a subsystem
// Parameters:
//     system: the system to update
//     enabled: true: enable file logging; false: disable file logging
// Returns:
//     None
fn log_set_system_file_logging_enabled_state(system: LogSystem, enabled: bool) {
    let system_name = critical_section(|cs| {
        let mut system_configs = LOG_SYSTEMS.borrow(cs).borrow_mut();

        for system_config in system_configs.iter_mut() {
            if system_config.system == system {
                system_config.file_logging_enabled = enabled;
                return Some(system_config.name);
            }
        }

        None
    });

    if let Some(name) = system_name {
        let mut message = String::<LOG_FORMATTED_MESSAGE_MAX_LENGTH>::new();
        let write_result = if enabled {
            write!(message, "Enabled file logging for {}", name)
        } else {
            write!(message, "Disabled file logging for {}", name)
        };

        if write_result.is_err() {
            return;
        }

        log_message(
            LogSystem::Log,
            LogSeverity::Normal,
            LOG_SINK_MASK_ALL,&message,);
    }
}


// Enable or disable debugging messages for a subsystem
// Parameters:
//     system: the system to update
//     enabled: true: enable debugging; false: disable debugging
// Returns:
//     None
fn log_set_system_debugging_messages_state(system: LogSystem, state: bool) {

    let mask = LogSeverity::Debug as u32;

    let system_name = critical_section(|cs| {
        let mut system_configs = LOG_SYSTEMS.borrow(cs).borrow_mut();

        for system_config in system_configs.iter_mut() {
            if system_config.system == system {
                if state {
                    system_config.severity_mask |= mask;
                } else {
                    system_config.severity_mask &= !mask;
                }

                return Some(system_config.name);
            }
        }

        None
    });

    if let Some(name) = system_name {
        let mut message = String::<LOG_FORMATTED_MESSAGE_MAX_LENGTH>::new();
        let write_result = if state {
            write!(message, "Enabled debugging messages for {}", name)
        } else {
            write!(message, "Disabled debugging messages for {}", name)
        };

        if write_result.is_err() {
            return;
        }

        log_message(
            LogSystem::Log,
            LogSeverity::Normal,
            LOG_SINK_MASK_ALL,
            &message,
        );
    }
}

//Retrieve the current logging configuration
fn log_get_configuration() -> LogConfiguration {
    critical_section(|cs| {
        LogConfiguration {
            sinks: *LOG_SINKS.borrow(cs).borrow(),
            systems: *LOG_SYSTEMS.borrow(cs).borrow(),
        }
    })
}

// Log the current configuration of all sinks and subsystems
fn log_report_configuration() {
    //Note: Currently sends each configuration as a seperate log meassage, not as one message
    let config = log_get_configuration();

    for sink in config.sinks.iter() {
        if let Some(message) = sink.format() {
            log_message(
                LogSystem::Log,
                LogSeverity::Normal,
                LOG_SINK_MASK_ALL,
                &message,
            );
        }
    }

    for system in config.systems.iter() {
        if let Some(message) = system.format() {
            log_message(
                LogSystem::Log,
                LogSeverity::Normal,
                LOG_SINK_MASK_ALL,
                &message,
            );
        }
    }
}


// TODO:implement radio logging
fn log_to_uhf_radio(_message: &str) {
}

// TODO:implement file system logging
fn log_to_file(_message: &str) {
}

// TODO: implement UART logging
fn log_to_uart(_message: &str) {
}