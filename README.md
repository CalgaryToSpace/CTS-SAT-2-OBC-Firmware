# CTS-SAT-2-OBC-Firmware
Firmware for the CTS-SAT-2 mission. Runs on the STM32L4R5ZI-based Onboard Computer. 

## Getting Started

1. [Install Rust](https://rust-lang.org/tools/install/). It must be installed using `rustup`.
2. Install the required dependencies for embedded rust development.

```bash
rustup update
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools
cargo install cargo-binutils probe-rs-tools cargo-expand just
```

3. Open this repo in VS Code.
4. To flash and run the firmware, run `cargo embed --target thumbv7em-none-eabihf` from the root of this repo, with the Nucleo-L4A6ZG plugged in.
5. Observe logs coming from the STM32. Observe the green onboard LED blinking.


## Command Quick Reference

* Run tests on host machine:

```sh
cargo test -p cts2_obc_logic
```

* Build and flash firmware to the OBC:

```sh
cargo embed --target thumbv7em-none-eabihf
```

* Build, test, and flash firmware at once:

```sh
cargo test -p cts2_obc_logic && cargo embed --target thumbv7em-none-eabihf
```
