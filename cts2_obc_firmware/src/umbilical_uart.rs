use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use rtt_target::rprintln;
use stm32l4xx_hal::{self as stm32_hal};

use crate::telecommands::exec::executor::{DispatchOutcome, dispatch_command};

/// Maximum length of a telecommand string received over the umbilical UART.
/// Includes the length of the command name, arguments, terminating newline, etc.
pub const MAX_TELECOMMAND_STR_LENGTH: usize = 256;

// Need an extra byte to hold a complete 256-byte command
const UART_BUF_SIZE: usize = MAX_TELECOMMAND_STR_LENGTH + 1;
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

// Examine next byte without popping it from the buffer
fn uart_peek_byte() -> Option<u8> {
    let tail = UART_TAIL.load(Ordering::Relaxed);
    let head = UART_HEAD.load(Ordering::Acquire);

    if tail == head {
        None
    } else {
        Some(UART_RX_BUF[tail].load(Ordering::Acquire))
    }
}

/// Process commands received over the umbilical UART, from the `UART_RX_BUF`.
pub fn process_umbilical_commands() {
    loop {
        // Ignore line endings outside commands.
        while matches!(uart_peek_byte(), Some(b'\r' | b'\n')) {
            uart_pop_byte();
        }

        let tail = UART_TAIL.load(Ordering::Acquire);
        let head = UART_HEAD.load(Ordering::Acquire);

        if tail == head {
            return;
        }

        // Look for the '!' terminator before consuming
        // any bytes from the circular buffer.
        let mut pos = tail;
        let mut complete = false;

        while pos != head {
            if UART_RX_BUF[pos].load(Ordering::Acquire) == b'!' {
                complete = true;
                break;
            }

            pos = (pos + 1) % UART_BUF_SIZE;
        }

        // Keep incomplete commands in the buffer.
        if !complete {
            // Discard data if an incomplete command
            // has filled the entire circular buffer.
            if (head + 1) % UART_BUF_SIZE == tail {
                rprintln!("UART command too long; discarding");

                while uart_pop_byte().is_some() {}
            }

            return;
        }

        let mut cmd = [0u8; MAX_TELECOMMAND_STR_LENGTH];
        let mut idx = 0;
        let mut too_long = false;

        // We now know a complete command is available.
        while let Some(b) = uart_pop_byte() {
            if idx < cmd.len() {
                cmd[idx] = b;
                idx += 1;
            } else {
                too_long = true;
            }

            if b == b'!' {
                break;
            }
        }

        if too_long {
            rprintln!("UART command exceeds maximum length");
            continue;
        }

        // Convert the received bytes into a string.
        match core::str::from_utf8(&cmd[..idx]) {
            Ok(cmd_str) => {
                rprintln!("CMD: {}", cmd_str);

                match dispatch_command(cmd_str) {
                    Ok(DispatchOutcome::Executed) => {
                        rprintln!("Command executed successfully");
                    }

                    Ok(DispatchOutcome::Duplicate) => {
                        rprintln!("Duplicate command ignored");
                    }

                    Ok(DispatchOutcome::DelayedNotReady) => {
                        rprintln!("Delayed execution not available");
                    }

                    Err(_) => {
                        rprintln!("Command execution failed");
                    }
                }
            }

            Err(_) => {
                rprintln!("Invalid UTF-8 received");
            }
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
