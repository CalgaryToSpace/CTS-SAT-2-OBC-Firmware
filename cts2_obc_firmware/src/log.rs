use core::fmt::Write;
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

#[derive(Copy, Clone)]
#[repr(u32)]
enum LogSink {
    UhfRadio = 1 << 0,
    File = 1 << 1,
    UmbilicalUart = 1 << 2,
}

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

struct LogSinkConfig {
    sink: LogSink,
    name: &'static str,
    enabled: bool,
    severity_mask: u32,
}

//sink configurations
static LOG_SINKS: [LogSinkConfig; 3] = [
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
];

// Log severity masks for filtering log messages
const LOG_SEVERITY_MASK_ALL: u32 =
    (LogSeverity::Debug as u32) | (LogSeverity::Normal as u32) | (LogSeverity::Warning as u32) | (LogSeverity::Error as u32) | (LogSeverity::Critical as u32);
    
const LOG_SEVERITY_MASK_ALL_EXCEPT_DEBUG: u32 = LOG_SEVERITY_MASK_ALL & !(LogSeverity::Debug as u32);
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

//filters and sends log message to appropriate sink
fn log_message(system: LogSystem,severity: LogSeverity,sink_mask: u32,message: &str){
    // Format log message
    let formatted_message = match format_log_message(&system, &severity, message) {
        Some(message) => message,
        None => return,
    };

    for sink_config in LOG_SINKS.iter() {
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
                log_to_file(&formatted_message);
            }
            LogSink::UmbilicalUart => {
                log_to_uart(&formatted_message);
            }
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