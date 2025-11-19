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
3. Install [SerialTest](https://github.com/wh201906/SerialTest/releases) or a similar serial terminal tool (must allow pre-rewriting a message before sending).

4. Open this repo in VS Code.
5. To flash and run the firmware, run `cargo embed --target thumbv7em-none-eabihf` from the root of this repo, with the Nucleo-L4A6ZG plugged in.
6. Observe logs coming from the STM32. Observe the green onboard LED blinking and logs in the debug terminal.
7. Disconnect power. Connect the USB-UART converter to the OBC's UART2 port.
    * Connect RX to TX, TX to RX, GND to GND.
    * Google "nucleo-144 pinout" to find the UART2 pin locations.
    * Ask a friend for help!
8. Open SerialTest (or similar) and connect to the appropriate COM port at 115200 baud.
    * You should see heartbeat messages every second.
9. Try sending the "PING" command. You should receive a "PONG" response.


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
