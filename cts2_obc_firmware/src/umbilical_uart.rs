use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use cts2_obc_telecommands::{Telecommand, parse_telecommand};
use rtt_target::rprintln;
use stm32l4xx_hal::{self as stm32_hal};

use crate::telecommand_implementation::demo_commands::run_hello_world_telecommand;

/// Maximum length of a telecommand string received over the umbilical UART.
/// Includes the length of the command name, arguments, terminating newline, etc.
pub const MAX_TELECOMMAND_STR_LENGTH: usize = 256;

const UART_BUF_SIZE: usize = MAX_TELECOMMAND_STR_LENGTH;
static UART_RX_BUF: [AtomicU8; UART_BUF_SIZE] = [const { AtomicU8::new(0) }; UART_BUF_SIZE];
static UART_HEAD: AtomicUsize = AtomicUsize::new(0);
static UART_TAIL: AtomicUsize = AtomicUsize::new(0);

/// Poll the UART RX DMA circular buffer and push received bytes into `UART_RX_BUF`.
///
/// This function should be called periodically to process incoming UART data, from the
/// main superloop or similar.
pub fn poll_uart_rx(
    rx_transfer: &mut stm32_hal::dma::CircBuffer<
        [u8; MAX_TELECOMMAND_STR_LENGTH],
        stm32_hal::dma::RxDma<
            stm32_hal::serial::Rx<stm32_hal::pac::USART2>,
            stm32_hal::dma::dma1::C6,
        >,
    >,
) {
    let mut buf = [0; MAX_TELECOMMAND_STR_LENGTH];
    let buf_size = rx_transfer.read(&mut buf).unwrap();

    // Process data[..pending].
    for &b in buf.iter().take(buf_size) {
        if b != 0 {
            rprintln!("RX: {}", b);
        }
        uart_push_byte(b);
    }
}

/// Push a byte into `UART_RX_BUF` and update `UART_HEAD`.
fn uart_push_byte(b: u8) {
    let head = UART_HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % UART_BUF_SIZE;
    if next != UART_TAIL.load(Ordering::Acquire) {
        UART_RX_BUF[head].store(b, Ordering::Release);
        UART_HEAD.store(next, Ordering::Release);
    } else {
        rprintln!("UART RX buffer overflow, dropping byte {}", b);
    }
}

/// If available, fetch a byte from `UART_RX_BUF`. Returns `None` if buffer is empty.
fn uart_pop_byte() -> Option<u8> {
    let mut byte = None;

    let tail = UART_TAIL.load(Ordering::Relaxed);
    let head = UART_HEAD.load(Ordering::Acquire);
    if tail != head {
        byte = Some(UART_RX_BUF[tail].load(Ordering::Acquire));
        UART_TAIL.store((tail + 1) % UART_BUF_SIZE, Ordering::Release);
    }

    byte
}

/// Process commands received over the umbilical UART, from the `UART_RX_BUF`.
pub fn process_umbilical_commands() {
    let mut cmd = [0u8; MAX_TELECOMMAND_STR_LENGTH];
    let mut idx = 0;

    rprintln!(
        "Processing UART commands. HEAD={}, TAIL={}",
        UART_HEAD.load(Ordering::Relaxed),
        UART_TAIL.load(Ordering::Relaxed)
    );

    while let Some(b) = uart_pop_byte() {
        if b == b'\n' || idx >= cmd.len() {
            if idx > 0 {
                if let Ok(cmd_str) = core::str::from_utf8(&cmd[..idx]) {
                    let trimmed = cmd_str.trim_end();
                    rprintln!("CMD: {}", trimmed);
                    match dispatch_command(trimmed) {
                        Ok(_) => rprintln!("Command executed successfully"),
                        Err(_) => rprintln!("Command execution failed"),
                    }
                }
                idx = 0;
            }
        } else {
            cmd[idx] = b;
            idx += 1;
        }
    }
}

// TODO: Make different functions to handle each separate command.
// TODO: Fix the () error type to be enum or string
// TODO: Replace with meaningful telecommands.
fn dispatch_command(cmd_str: &str) -> Result<(), ()> {
    let cmd = parse_telecommand(cmd_str);
    match cmd {
        Ok(Telecommand::HelloWorld) => run_hello_world_telecommand(),
        Ok(Telecommand::DemoCommandWithArguments(args)) => {
            crate::telecommand_implementation::demo_commands::run_demo_command_with_arguments(args)
        }
        Err(e) => {
            send_umbilical_uart(b"ERR: unknown command\r\n");
            Err(e)
        }
    }
}

/// Send data over the umbilical UART (e.g., as a response to a command).
///
/// Blocks during transmission.
pub fn send_umbilical_uart(data: &[u8]) {
    let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };
    for &b in data {
        while usart2.isr.read().txe().bit_is_clear() {}
        usart2.tdr.write(|w| w.tdr().bits(b as u16));
    }
}
