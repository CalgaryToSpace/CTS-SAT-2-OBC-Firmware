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

mod tcmd_uart;

use tcmd_uart::{process_uart_commands, send_uart2};

type Usart2Serial = stm32_hal::serial::Serial<
    stm32_hal::stm32::USART2,
    (
        stm32_hal::gpio::gpiod::PD5<stm32_hal::gpio::Alternate<PushPull, 7>>,
        stm32_hal::gpio::gpiod::PD6<stm32_hal::gpio::Alternate<PushPull, 7>>,
    ),
>;

static PERIPHERAL_GREEN_LED: Mutex<RefCell<Option<PC7<Output<PushPull>>>>> =
    Mutex::new(RefCell::new(None));

static PERIPHERAL_DELAY_TIMER: Mutex<RefCell<Option<stm32_hal::delay::Delay>>> =
    Mutex::new(RefCell::new(None));

static PERIPHERAL_RCC: Mutex<RefCell<Option<stm32_hal::rcc::Rcc>>> = Mutex::new(RefCell::new(None));
static PERIPHERAL_CLOCKS: Mutex<RefCell<Option<stm32_hal::rcc::Clocks>>> =
    Mutex::new(RefCell::new(None));

static UART_UMBILICAL_RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();

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
    let led = gpioc
        .pc7
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    // --- Move peripherals into global statics ---
    critical_section(|cs| {
        PERIPHERAL_GREEN_LED.borrow(cs).replace(Some(led));
        PERIPHERAL_DELAY_TIMER.borrow(cs).replace(Some(timer));
    });

    // --- USART2 Setup ---
    let (rx_dma, _tx_dma) = {
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

        let (tx, rx) = serial.split();

        let dma_channels = peripheral.DMA1.split(&mut rcc.ahb1);

        let rx_dma = rx.with_dma(dma_channels.6);
        let tx_dma = tx.with_dma(dma_channels.7);
        (rx_dma, tx_dma)
    };

    rprintln!("Starting DMA-based UART RX...");

    let buf: &'static mut [u8; 256] = UART_UMBILICAL_RX_BUF.init([0; 256]); // Initialize once at startup.
    let mut rx_transfer = rx_dma.circ_read(buf);

    // Enable FIFO mode and set RX FIFO threshold
    // let usart2 = unsafe { &*stm32_hal::stm32::USART2::ptr() };
    // usart2.cr1.modify(|_, w| w.fifoen().set_bit()); // Enable FIFO
    // usart2.cr3.modify(|_, w| w.rxftie().set_bit()); // RX FIFO threshold interrupt
    // usart2.cr3.modify(|_, w| w.rxftcfg().bits(0b010)); // 1/4 full threshold (for example)

    // serial.listen(stm32_hal::serial::Event::Rxne);

    // critical_section(|cs| {
    //     PERIPHERAL_USART_2.borrow(cs).replace(Some(serial));
    // });
    rprintln!("USART2 initialized for 115200 8N1.");

    unsafe {
        NVIC::unmask(stm32_hal::stm32::Interrupt::USART2);
    }

    send_uart2(b"USART2 ready. Buffered RX active.\r\n");

    // --- Main loop ---
    let mut i = 0u32;
    loop {
        toggle_led();

        poll_uart_rx(&mut rx_transfer);

        // Periodically check for incoming commands
        process_uart_commands();

        // Heartbeat message
        rprintln!("Heartbeat {}", i);
        send_uart2(b"HEARTBEAT\r\n");

        timer_delay_ms(500_u16);
        i = i.wrapping_add(1);
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

// fn poll_uart_rx(rx_transfer: &CircBuffer<[u8; 256], dma::dma1::C6>) {
fn poll_uart_rx(
    rx_transfer: &mut stm32l4xx_hal::dma::CircBuffer<
        [u8; 256],
        stm32l4xx_hal::dma::RxDma<
            stm32l4xx_hal::serial::Rx<stm32l4xx_hal::pac::USART2>,
            stm32l4xx_hal::dma::dma1::C6,
        >,
    >,
) {
    let mut buf = [0; 256];
    let xx = rx_transfer.read(&mut buf).unwrap();
    // let (buf, pending) = rx_transfer.peek(|data, _| (data, 0));
    // Process data[..pending]
    for &b in buf.iter().take(xx) {
        if b != 0 {
            rprintln!("RX: {}", b);
        }
    }
}
