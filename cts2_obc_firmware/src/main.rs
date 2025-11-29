#![no_std]
#![no_main]

extern crate cortex_m;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free as critical_section;
use cortex_m::peripheral::NVIC;
use rtt_target::{rprintln, rtt_init_print};
use static_cell::StaticCell;
use stm32l4xx_hal::{
    self as stm32_hal,
    dma::CircReadDma as _,
    gpio::{Output, PushPull, gpioc::PC7},
    prelude::*,
};
// SPI traits + pin traits from the HAL's re-exported embedded-hal
use stm32_hal::hal::blocking::spi::{Transfer, Write};
use stm32_hal::hal::digital::v2::OutputPin;
use stm32_hal::hal::spi::{Mode, Phase, Polarity};

// STM32L4 SPI peripheral type
use stm32_hal::spi::Spi;







mod umbilical_uart;

use umbilical_uart::{process_umbilical_commands, send_umbilical_uart};

use crate::umbilical_uart::poll_uart_rx;

static PERIPHERAL_GREEN_LED: Mutex<RefCell<Option<PC7<Output<PushPull>>>>> =
    Mutex::new(RefCell::new(None));

static PERIPHERAL_DELAY_TIMER: Mutex<RefCell<Option<stm32_hal::delay::Delay>>> =
    Mutex::new(RefCell::new(None));

static PERIPHERAL_RCC: Mutex<RefCell<Option<stm32_hal::rcc::Rcc>>> = Mutex::new(RefCell::new(None));
static PERIPHERAL_CLOCKS: Mutex<RefCell<Option<stm32_hal::rcc::Clocks>>> =
    Mutex::new(RefCell::new(None));

static UART_DMA_UMBILICAL_RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();

const CMD_RESET: u8   = 0xFF; // RESET command from datasheet
const CMD_READ_ID: u8 = 0x9F; // READ ID command from datasheet


#[cortex_m_rt::entry]
fn entry_point() -> ! {
    rtt_init_print!();
    rprintln!("System startup...");

    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripheral = stm32_hal::stm32::Peripherals::take().unwrap();

    // --- Clock setup ---
    critical_section(|cs| {
        PERIPHERAL_RCC
            .borrow(cs)
            .replace(Some(peripheral.RCC.constrain()));
    });

    let mut flash = peripheral.FLASH.constrain();
    let mut rcc = critical_section(|cs| PERIPHERAL_RCC.borrow(cs).borrow_mut().take().unwrap());
    let mut pwr = peripheral.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc.cfgr.sysclk(64.MHz()).freeze(&mut flash.acr, &mut pwr);
    critical_section(|cs| {
        PERIPHERAL_CLOCKS.borrow(cs).replace(Some(clocks));
    });
    rprintln!("Clocks configured.");

    let timer = stm32_hal::delay::Delay::new(cortex_peripherals.SYST, clocks);

// --- GPIO ---
let mut gpioc = peripheral.GPIOC.split(&mut rcc.ahb2);
let mut gpiod = peripheral.GPIOD.split(&mut rcc.ahb2);
let mut gpioa = peripheral.GPIOA.split(&mut rcc.ahb2);
// (Only add GPIOB if you actually use it; can be removed otherwise.)

let led = gpioc
    .pc7
    .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

// SPI1 pins on Arduino headers, Nucleo-L4R5ZI mapping:
// D13 = PA5 = SCK
// D12 = PA6 = MISO
// D11 = PA7 = MOSI
let sck = gpioa
    .pa5
    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
let miso = gpioa
    .pa6
    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
let mosi = gpioa
    .pa7
    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

// Chip select on D10 = PD14
let mut nand_cs = gpiod
    .pd14
    .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
nand_cs.set_high(); // idle high (chip not selected)




    let spi_mode = Mode {
        polarity: Polarity::IdleLow,                 // SPI mode 0
        phase: Phase::CaptureOnFirstTransition,
    };

    let mut spi1 = Spi::spi1(
        peripheral.SPI1,
        (sck, miso, mosi),
        spi_mode,
        10.MHz(),
        clocks,
        &mut rcc.apb2,
    );

    rprintln!("SPI1 configured for NAND.");

    // Probe the NAND
    test_nand(&mut spi1, &mut nand_cs);


    // --- Move peripherals into global statics ---
    critical_section(|cs| {
        PERIPHERAL_GREEN_LED.borrow(cs).replace(Some(led));
        PERIPHERAL_DELAY_TIMER.borrow(cs).replace(Some(timer));
    });

    // --- USART2 Setup ---
    let rx_dma = {
        let tx = gpiod
            .pd5
            .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);
        let rx = gpiod
            .pd6
            .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);

        let serial_cfg = stm32_hal::serial::Config::default().baudrate(115_200.bps());
        let serial = stm32_hal::serial::Serial::usart2(
            peripheral.USART2,
            (tx, rx),
            serial_cfg,
            clocks,
            &mut rcc.apb1r1,
        );

        let (_tx, rx) = serial.split();

        let dma_channels = peripheral.DMA1.split(&mut rcc.ahb1);

        rx.with_dma(dma_channels.6)
    };

    rprintln!("Starting DMA-based UART RX...");
    let mut rx_transfer = {
        let buf: &'static mut [u8; 256] = UART_DMA_UMBILICAL_RX_BUF.init([0; 256]); // Initialize once at startup.
        rx_dma.circ_read(buf)
    };
    rprintln!("USART2 initialized for 115200 8N1.");

    unsafe {
        NVIC::unmask(stm32_hal::stm32::Interrupt::USART2);
    }

    send_umbilical_uart(b"USART2 ready. Buffered RX active.\r\n");

    // --- Main loop ---
    let mut i = 0u32;
    loop {
        toggle_led();

        poll_uart_rx(&mut rx_transfer);

        // Periodically check for incoming commands
        process_umbilical_commands();

        // Heartbeat message
        rprintln!("Heartbeat {}", i);
        send_umbilical_uart(b"HEARTBEAT\r\n");

        timer_delay_ms(500_u16);
        i = i.wrapping_add(1);
    }
}
// testing nand 
fn test_nand<SPI, CS, E>(spi: &mut SPI, cs: &mut CS)
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin,
{
    rprintln!("Starting NAND test...");

    // ----- RESET -----
    // Pull CS low to select the chip
    cs.set_low();
    // Send single-byte RESET command 0xFF
    if let Err(_) = spi.write(&[CMD_RESET]) {
        rprintln!("SPI write failed (RESET)");
    }
    // Release the chip
    cs.set_high();

    // Wait for NAND to finish internal reset (tRST)
    // This is a crude busy-wait: number is "cycles at CPU clock"
    cortex_m::asm::delay(64_0000); // ≈10 ms at 64 MHz

   // ----- READ ID -----
let mut buf = [CMD_READ_ID, 0x00, 0x00, 0x00, 0x00, 0x00];
//             cmd       dummy  b2   b3   b4   b5

cs.set_low();
let res = spi.transfer(&mut buf); // full-duplex
cs.set_high();

match res {
    Ok(rx) => {
        rprintln!(
            "RAW RX: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            rx[0], rx[1], rx[2], rx[3], rx[4], rx[5]
        );
        let mfr = rx[2];
        let dev = rx[3];
        rprintln!("NAND ID Response: MFR=0x{:02X}, DEV=0x{:02X}", mfr, dev);
    }
    Err(_) => {
        rprintln!("SPI transfer failed (READ ID)");
    }
}

}


fn toggle_led() {
    critical_section(|cs| {
        if let Some(ref mut led) = *PERIPHERAL_GREEN_LED.borrow(cs).borrow_mut() {
            led.toggle();
        }
    });
}

fn timer_delay_ms(ms: u16) {
    critical_section(|cs| {
        if let Some(ref mut timer) = *PERIPHERAL_DELAY_TIMER.borrow(cs).borrow_mut() {
            timer.delay_ms(ms);
        }
    });
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
