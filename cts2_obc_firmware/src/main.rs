#![no_std]
#![no_main]

extern crate cortex_m;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free as critical_section;
use cortex_m::peripheral::NVIC;
use rtt_target::{rprintln, rtt_init_print};
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::{
    self as stm32_hal,
    gpio::{Output, PushPull, gpioc::PC7},
};

mod tcmd_uart;

use tcmd_uart::{process_uart_commands, send_uart2};

static PERIPHERAL_GREEN_LED: Mutex<RefCell<Option<PC7<Output<PushPull>>>>> =
    Mutex::new(RefCell::new(None));

static PERIPHERAL_DELAY_TIMER: Mutex<RefCell<Option<stm32_hal::delay::Delay>>> =
    Mutex::new(RefCell::new(None));

/// RCC = Reset and Control Clock.
static PERIPHERAL_RCC: Mutex<RefCell<Option<stm32_hal::rcc::Rcc>>> = Mutex::new(RefCell::new(None));

static PERIPHERAL_CLOCKS: Mutex<RefCell<Option<stm32_hal::rcc::Clocks>>> =
    Mutex::new(RefCell::new(None));

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
    {
        let tx = gpiod
            .pd5
            .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);
        let rx = gpiod
            .pd6
            .into_alternate(&mut gpiod.moder, &mut gpiod.otyper, &mut gpiod.afrl);

        let serial_cfg = stm32_hal::serial::Config::default().baudrate(115_200.bps());
        let mut serial = stm32_hal::serial::Serial::usart2(
            peripheral.USART2,
            (tx, rx),
            serial_cfg,
            clocks,
            &mut rcc.apb1r1,
        );

        serial.listen(stm32_hal::serial::Event::Rxne);
        rprintln!("USART2 initialized for 115200 8N1.");
    }

    unsafe {
        NVIC::unmask(stm32_hal::stm32::Interrupt::USART2);
    }

    send_uart2(b"USART2 ready. Buffered RX active.\r\n");

    // --- Main loop ---
    let mut i = 0u32;
    loop {
        toggle_led();

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
