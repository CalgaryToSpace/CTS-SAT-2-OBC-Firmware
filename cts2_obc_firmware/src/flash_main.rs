// src/flash_main.rs
use cortex_m;
use rtt_target::rprintln;
use stm32l4xx_hal as stm32_hal;

// Traits we need for SPI + CS
use stm32_hal::hal::blocking::spi::{Transfer, Write};
use stm32_hal::hal::digital::v2::OutputPin;

const CMD_RESET: u8   = 0xFF;
const CMD_READ_ID: u8 = 0x9F;
const CMD_READ_PAGE: u8 = 0x13;
/// Simple NAND driver that owns an SPI bus + chip-select pin.
pub struct Nand<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS, E> Nand<SPI, CS>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin,
{
    pub fn new(mut spi: SPI, mut cs: CS) -> Self {
        // Make sure CS is high (inactive)
        let _ = cs.set_high();
        rprintln!("NAND driver created.");
        Self { spi, cs }
    }

    pub fn read_id(&mut self) {
        rprintln!("Starting NAND READ ID...");

        // ----- READ ID -----
        let mut buf = [CMD_READ_ID, 0x00];

        let _ = self.cs.set_low();
        let res = self.spi.transfer(&mut buf);
        let _ = self.cs.set_high();

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
    pub fn reset(&mut self){
        // ----- RESET -----
        let _ = self.cs.set_low();
        if let Err(_) = self.spi.write(&[CMD_RESET]) {
            rprintln!("SPI write failed (RESET)");
        }
        let _ = self.cs.set_high();

        cortex_m::asm::delay(64_0000); 
    }


    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }
}
