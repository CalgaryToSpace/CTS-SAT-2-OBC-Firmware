use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use cts2_obc_telecommands::Telecommand;
use cts2_obc_telecommands::parse_command;
use rtt_target::rprintln;
use stm32l4xx_hal::{self as stm32_hal};
const UART_BUF_SIZE: usize = 256;
static UART_RX_BUF: [AtomicU8; UART_BUF_SIZE] = [const { AtomicU8::new(0) }; UART_BUF_SIZE];
static UART_HEAD: AtomicUsize = AtomicUsize::new(0);
static UART_TAIL: AtomicUsize = AtomicUsize::new(0);

/// Poll the UART RX DMA circular buffer and push received bytes into `UART_RX_BUF`.
///
/// This function should be called periodically to process incoming UART data, from the
/// main superloop or similar.
pub fn poll_uart_rx(
    rx_transfer: &mut stm32l4xx_hal::dma::CircBuffer<
        [u8; 256],
        stm32l4xx_hal::dma::RxDma<
            stm32l4xx_hal::serial::Rx<stm32l4xx_hal::pac::USART2>,
            stm32l4xx_hal::dma::dma1::C6,
        >,
    >,
) {
    let mut buf = [0; 256];
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
    let mut cmd = [0u8; 64];
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
                    handle_command(trimmed);
                }
                idx = 0;
            }
        } else {
            cmd[idx] = b;
            idx += 1;
        }
    }
}

// Creating a more modular structure so that handling commands will be easier when we have more commands.
// First I will parse the incoming string into a structured command, match on that strutured enum, and respond accordingly.

//Not sure if we would want to make a different function to handle each separate command?
fn handle_command(cmd_str: &str) -> Result<Telecommand, ()> {
    let cmd = parse_command(cmd_str);
    match cmd {
        Ok(Telecommand::Ping) => {
            send_umbilical_uart(b"PONG\r\n");
            Ok(Telecommand::Ping)
        }
        Ok(Telecommand::LedOn) => {
            //We would eventually want a command like this to do meaningful things (like actually turn the LED on.)
            send_umbilical_uart(b"LED ON\r\n");
            Ok(Telecommand::LedOn)
        }
        Ok(Telecommand::LedOff) => {
            send_umbilical_uart(b"LED OFF\r\n");
            Ok(Telecommand::LedOff)
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
