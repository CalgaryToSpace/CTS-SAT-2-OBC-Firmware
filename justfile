# CTS-SAT-2-OBC-Firmware Justfile
# Firmware for the CTS-SAT-2 mission, targeting STM32L4A6-based OBC.

# Set the target triple for ARM Cortex-M4F (STM32L4A6).
target := "thumbv7em-none-eabihf"

# Show available commands.
default:
    @just --list

# Show available commands.
help:
    @just --list

# Install dependencies for embedded Rust development.
setup:
    rustup update
    rustup target add {{target}}
    rustup component add llvm-tools
    cargo install cargo-binutils probe-rs-tools cargo-expand just

# Run host-side logic tests.
test:
    cargo test -p cts2_obc_logic
    cargo test -p cts2_obc_telecommands

# Build, flash, and run firmware on the STM32.
flash:
    cargo embed --target {{target}}

# Run tests on the host, then flash and run if successful.
test-flash:
    just test && just flash
