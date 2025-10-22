#![no_std]
#![no_main]

extern crate cortex_m;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use rtt_target::{rprintln, rtt_init_print};
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::{self as stm32_hal, interrupt};

// Single-byte "mailbox" used by the IRQ to buffer one pending TX byte when TXE isn't ready.
static PENDING_VALID: AtomicBool = AtomicBool::new(false);
static PENDING_BYTE: AtomicU8 = AtomicU8::new(0);

#[cortex_m_rt::entry]
fn entry_point() -> ! {
    rtt_init_print!();
    rprintln!("rtt_init_print() done.");

    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripheral = stm32_hal::stm32::Peripherals::take().unwrap();

    // Clocks / power
    let mut rcc = peripheral.RCC.constrain();
    let mut flash = peripheral.FLASH.constrain();
    let mut pwr = peripheral.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc.cfgr.sysclk(64.MHz()).freeze(&mut flash.acr, &mut pwr);
    rprintln!("Clocks are configured.");

    // Systick-based delay
    let mut timer = stm32_hal::delay::Delay::new(cortex_peripherals.SYST, clocks);

    // GPIO
    let mut _gpioa = peripheral.GPIOA.split(&mut rcc.ahb2);
    let mut gpioc = peripheral.GPIOC.split(&mut rcc.ahb2);
    let mut gpiod = peripheral.GPIOD.split(&mut rcc.ahb2);
    let mut led = gpioc
        .pc7
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    // -------------------------
    // USART2 @ 115_200 8N1
    // Pins: PD5 (TX), PD6 (RX), AF7 (alternate function 7)
    // -------------------------
    let tx = gpiod
        .pd5
        .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);
    let rx = gpiod
        .pd6
        .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);

    // Configure and enable the peripheral
    let serial_cfg = stm32_hal::serial::Config::default().baudrate(115_200.bps());
    let mut serial = stm32_hal::serial::Serial::usart2(
        peripheral.USART2,
        (tx, rx),
        serial_cfg,
        clocks,
        &mut rcc.apb1r1,
    );

    // Enable RXNE interrupt (byte received).
    serial.listen(stm32_hal::serial::Event::Rxne);

    // Unmask USART2 IRQ in NVIC (enable interrupt).
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32_hal::stm32::Interrupt::USART2);
    }

    rprintln!("USART2 up at 115200 8N1. Interrupt-driven echo is running…");
    send_uart2("USART2 up at 115200 8N1. Interrupt-driven echo is running...\r\n".as_bytes());

    // Heartbeat LED.
    let mut i: u32 = 0;
    loop {
        led.toggle();
        i = i.wrapping_add(1);
        if (i & 0x01) == 0 {
            rprintln!("i = {}", i);
        }
        timer.delay_ms(1000_u16);
    }
}

// Interrupt handler for USART2
#[interrupt]
fn USART2() {
    // Access the raw peripheral safely via the PAC pointer.
    let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };

    let isr = usart2.isr.read();

    // --- RXNE: a byte has arrived.
    if isr.rxne().bit_is_set() {
        // Reading RDR clears RXNE.
        let b = usart2.rdr.read().rdr().bits() as u8;

        // Try to echo immediately if TXE (TDR empty).
        if usart2.isr.read().txe().bit_is_set() {
            // Writing TDR clears TXE.
            usart2.tdr.write(|w| w.tdr().bits(b as u16));
        } else {
            // Buffer one byte and enable TXE interrupt to flush later.
            PENDING_BYTE.store(b, Ordering::Relaxed);
            PENDING_VALID.store(true, Ordering::Release);
            usart2.cr1.modify(|_, w| w.txeie().set_bit());
        }
    }

    // --- TXE: transmit holding register is empty (ready to accept a byte).
    if isr.txe().bit_is_set() {
        if PENDING_VALID.swap(false, Ordering::AcqRel) {
            let b = PENDING_BYTE.load(Ordering::Relaxed);
            usart2.tdr.write(|w| w.tdr().bits(b as u16));
            // After writing, TXE will clear automatically. If there is no more data pending,
            // we can disable TXEIE to avoid spurious interrupts.
            if !PENDING_VALID.load(Ordering::Acquire) {
                usart2.cr1.modify(|_, w| w.txeie().clear_bit());
            }
        } else {
            // Nothing to send. Make sure TXEIE is off.
            usart2.cr1.modify(|_, w| w.txeie().clear_bit());
        }
    }

    // Optional: handle other flags (ORE/FE/NE/IDLE) for more robustness.
    // e.g., clear ORE by reading RDR when ORE set, etc.
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

fn send_uart2(data: &[u8]) {
    let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };

    for &b in data {
        // Wait until TXE is set (TDR empty).
        while usart2.isr.read().txe().bit_is_clear() {}
        usart2.tdr.write(|w| w.tdr().bits(b as u16));
    }
}
