# Logging

## Overview
The logging system records events from different satellite subsystems, allowing their behavior to be monitored and analyzed, particularly when errors or unexpected events occur.

### Logging architecture

- **Subsystem:** The source of a log message, such as EPS, GNSS, or ADCS.
- **Logger:** The implementation in `cts2_obc_firmware/src/log.rs`, which formats, filters, and routes messages.
- **Sink:** The destination where a log message is sent/stored.

The logger separates message generation from delivery. Each subsystem calls log_message() to record events, while the logger handles formatting, filtering, and routing.

**Note: actual sink delivery is not implemented yet*

## Using `log_message()`

The main logging function is:

```rust
log_message(system, severity, sink_mask, message);
```
- `system`: The subsystem generating the message
- `severity`: The severity level of the event
- `sink_mask`: A bitmask specifying the requested output sinks
- `message`: The message text

For example:

```rust
log_message(
    LogSystem::Eps,
    LogSeverity::Error,
    LOG_SINK_MASK_ALL,
    "Failed to do something",
);
```

This creates an error log from the EPS subsystem and requests delivery to all sinks.

The logger then checks the current configuration of the sinks to determine which destinations are currently able to receive the message.

## `log_message()` arguments

### Subsystems

The currently defined subsystems are:
```rust
LogSystem::Obc,
LogSystem::UhfRadio,
LogSystem::UmbilicalUart,
LogSystem::Gnss,
LogSystem::Mpi,
LogSystem::Eps,
LogSystem::Boom,
LogSystem::Adcs,
LogSystem::Lfs,
LogSystem::Flash,
LogSystem::AntennaDeploy,
LogSystem::Log,
LogSystem::Telecommand,
LogSystem::UnitTest,
LogSystem::Unknown,
```

Each subsystem has an independent configuration:

```rust

struct LogSystemConfig {
    system: LogSystem,
    name: &'static str,
    log_file_path: &'static str,
    file_logging_enabled: bool,
    severity_mask: u32,
}
```

### Severity levels

Each log message is assigned a severity level indicating the type of event being logged. Severity levels allow messages to be filtered based on their importance.

```rust
enum LogSeverity {
    Debug = 1 << 0,
    Normal = 1 << 1,
    Warning = 1 << 2,
    Error = 1 << 3,
    Critical = 1 << 4,
}
```

- `DEBUG`: detailed diagnostic information for debugging and testing
- `NORMAL`: routine operations
- `WARNING`: unexpected or abnormal system behavior
- `ERROR`: a failure that prevents an operation from completing as expected
- `CRITICAL`: severe faults that may compromise subsystem

### Sink Mask

A single message can be routed to multiple sinks. 


- `UHF_RADIO`: transmits log messages over the radio link to the ground station
- `FILE`: stores log messages in the filesystem for later retrieval
- `UMBILICAL_UART`: physically connected debugging interface for development and testing


```rust
enum LogSink {
    UhfRadio = 1 << 0,
    File = 1 << 1,
    UmbilicalUart = 1 << 2,
}
```

Each sink has an independent configuration:

```rust
struct LogSinkConfig {
    sink: LogSink,
    name: &'static str,
    enabled: bool,
    severity_mask: u32,
}
```

#### Selecting sinks

By default, `LOG_SINK_MASK_ALL` can be used to request delivery to all three sinks.

To request a specific sink, use its corresponding enum value:

```rust
log_message(
    LogSystem::Eps,
    LogSeverity::Warning,
    LogSink::UhfRadio as u32,
    "EPS battery voltage low",
);
```

Multiple sinks can be selected using the bitwise OR (`|`) operator:

```rust
log_message(
    LogSystem::Gnss,
    LogSeverity::Error,
    (LogSink::UhfRadio as u32) | (LogSink::File as u32),
    "Failed to read GNSS data",
);
```

This requests delivery to both the radio and file sinks.

A sink can also be excluded from the full mask using bitwise AND (`&`) and NOT (`!`):

```rust
let sinks = LOG_SINK_MASK_ALL & !(LogSink::File as u32);
```

This selects all sinks except FILE, which may be useful when logging a filesystem failure.

Selecting a sink does not guarantee delivery. The logger also checks whether the sink is enabled and configured to accept the message's severity.

### How messages are filtered and routed

When `log_message()` is called, the logger:

1. Checks whether the originating subsystem permits the message's severity. (e.g. if `DEBUG` messages from `GNSS` are disabled, the message won't be logged)
2. Formats the message with its subsystem name and severity.
3. Identifies the requested destinations using the sink mask.
4. Checks whether each selected sink is enabled and permits the message's severity.
5. Routes the message to eligible sinks. For `FILE`, file logging must also be enabled for the originating subsystem.

If a message fails the subsystem's severity filter, it is discarded. If an individual sink fails its checks, that sink is skipped without affecting the others.

## Runtime configuration

The logging system supports runtime configuration at two levels:
- Per-sink: Enable or disable individual sinks.
- Per-subsystem: Configure which severity levels can be sent from this subsystem, enable or disable DEBUG messages, and file logging for individual subsystems.

Sink and subsystem configurations can be updated and retrieved at runtime through telecommands.

## Log output format

The logger formats messages using the following structure:

```text
[SYSTEM:SEVERITY]: message
```

Examples:

```text
[TELECOMMAND:NORMAL]: Command received and validated
[EPS:WARNING]: EPS battery voltage low
[GNSS:ERROR]: Failed to read GNSS data
```

**Note: The logger currently supports formatted messages up to 185 bytes, including the `[SYSTEM:SEVERITY]:`  prefix and message text. The available space for the message text depends on the length of the prefix. If the combined length exceeds 185 bytes, formatting fails and the message is discarded.*