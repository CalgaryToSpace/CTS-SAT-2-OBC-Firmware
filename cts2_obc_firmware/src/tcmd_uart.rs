use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use cortex_m::interrupt::free as critical_section;
use rtt_target::rprintln;
use stm32l4xx_hal::{self as stm32_hal, interrupt};

const UART_BUF_SIZE: usize = 256;
static UART_RX_BUF: [AtomicU8; UART_BUF_SIZE] = [const { AtomicU8::new(0) }; UART_BUF_SIZE];
static UART_HEAD: AtomicUsize = AtomicUsize::new(0);
static UART_TAIL: AtomicUsize = AtomicUsize::new(0);

static PENDING_VALID: AtomicBool = AtomicBool::new(false);
static PENDING_BYTE: AtomicU8 = AtomicU8::new(0);

#[interrupt]
fn USART2() {
    let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };
    let isr = usart2.isr.read();

    // RXNE: byte received
    if isr.rxne().bit_is_set() {
        let b = usart2.rdr.read().rdr().bits() as u8;
        uart_push_byte(b);
    }

    // TXE: transmit empty
    if isr.txe().bit_is_set() {
        if PENDING_VALID.swap(false, Ordering::AcqRel) {
            let b = PENDING_BYTE.load(Ordering::Relaxed);
            usart2.tdr.write(|w| w.tdr().bits(b as u16));
            if !PENDING_VALID.load(Ordering::Acquire) {
                usart2.cr1.modify(|_, w| w.txeie().clear_bit());
            }
        } else {
            usart2.cr1.modify(|_, w| w.txeie().clear_bit());
        }
    }
}

fn uart_push_byte(b: u8) {
    critical_section(|_| {
        let head = UART_HEAD.load(Ordering::Relaxed);
        let next = (head + 1) % UART_BUF_SIZE;
        if next != UART_TAIL.load(Ordering::Acquire) {
            UART_RX_BUF[head].store(b, Ordering::Release);
            UART_HEAD.store(next, Ordering::Release);
        }
    });
}

fn uart_pop_byte() -> Option<u8> {
    let mut byte = None;
    critical_section(|_| {
        let tail = UART_TAIL.load(Ordering::Relaxed);
        let head = UART_HEAD.load(Ordering::Acquire);
        if tail != head {
            byte = Some(UART_RX_BUF[tail].load(Ordering::Acquire));
            UART_TAIL.store((tail + 1) % UART_BUF_SIZE, Ordering::Release);
        }
    });
    byte
}

pub fn process_uart_commands() {
    let mut cmd = [0u8; 64];
    let mut idx = 0;

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

fn handle_command(cmd: &str) {
    match cmd {
        "PING" => send_uart2(b"PONG\r\n"),
        "LED ON" => send_uart2(b"LED ON\r\n"),
        "LED OFF" => send_uart2(b"LED OFF\r\n"),
        _ => send_uart2(b"ERR: Unknown command\r\n"),
    }
}

pub fn send_uart2(data: &[u8]) {
    let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };
    for &b in data {
        while usart2.isr.read().txe().bit_is_clear() {}
        usart2.tdr.write(|w| w.tdr().bits(b as u16));
    }
}
