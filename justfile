# CTS-SAT-2-OBC-Firmware Justfile
# Firmware for the CTS-SAT-2 mission, targeting STM32L4A6ZG-based OBC.

# Set the target triple for ARM Cortex-M4F (STM32L4A6ZG)
target := "thumbv7em-none-eabihf"

# Default recipe: show available commands
default:
    @just --list

# Install dependencies for embedded Rust development
setup:
    rustup update
    rustup target add {{target}}
    rustup component add llvm-tools
    cargo install cargo-binutils probe-rs-tools cargo-expand just

# Run host-side logic tests
test:
    cargo test -p cts2_obc_logic
    cargo test -p cts2_obc_telecommands

# Build and flash firmware to the STM32 OBC
flash:
    cargo embed --target {{target}}

# Run tests, then flash if successful
test-flash:
    just test && just flash

# Open repository in VS Code
open:
    code .

# Observe logs from STM32 (adjust port as needed)
logs:
    probe-rs-cli run --chip STM32L4A6ZG
