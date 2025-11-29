# CTS-SAT-2-OBC-Firmware Justfile
# Firmware for the CTS-SAT-2 mission, targeting STM32L4-based OBC.

# Set the target triple for ARM Cortex-M4F (STM32L4).
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

# Run the Clippy linter.
check:
    # Check the entire workspace for the embedded target. Does not/may not check tests though.
    cargo clippy --workspace --target {{target}} --all-features -- -D warnings

    # Check the packages that build on all targets. Checks tests.
    cargo clippy -p cts2_obc_logic --all-features
    cargo clippy -p cts2_obc_telecommands --all-features

# Format the code using rustfmt.
format:
    cargo fmt --all

# Build, flash, and run firmware on the STM32.
flash:
    cargo embed --target {{target}}

# Run tests on the host, then flash and run if successful.
test-flash:
    just test && just flash

# Show how much flash space the firmware uses.
size:
    @echo "Recall, the STM32L4R5 has 2 MiB of flash and 640 KiB of RAM."
    @echo "Understand: RAM_usage = size(.data) + size(.bss) + STACK_SIZE + HEAP_SIZE"
    @echo ""
    @echo "DEBUG BUILD: Flash and RAM usage:"
    cargo size --target {{target}}
    @echo ""
    @echo "RELEASE BUILD: Flash and RAM usage:"
    cargo size --target {{target}} --release
    