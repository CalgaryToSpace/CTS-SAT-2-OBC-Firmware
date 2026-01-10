use cortex_m;
use rtt_target::rprintln;
use stm32l4xx_hal as stm32_hal;

// Traits we need for SPI + CS
use stm32_hal::hal::blocking::spi::{Transfer, Write};
use stm32_hal::hal::digital::v2::OutputPin;

const CMD_RESET: u8 = 0xFF;
const CMD_READ_ID: u8 = 0x9F;
const CMD_READ_PAGE: u8 = 0x13;
// NAND struct that holds the SPI bus and CS pins
pub struct Nand<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS, SpiE, CsE, E> Nand<SPI, CS>
where
    SPI: Write<u8, Error = SpiE> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = CsE>,
{
    pub fn new(mut spi: SPI, mut cs: CS) -> Result<Self, CsE>{
        cs.set_high()?;
        rprintln!("NAND driver created.");
        Ok(Self { spi, cs })
    }

    pub fn read_id(&mut self) -> Result<[u8; 6], E>

        rprintln!("Starting NAND READ ID...");

        // ----- READ ID -----
        let mut buf = [CMD_READ_ID, 0x00];

        self.cs.set_low()?;
        let res = self.spi.transfer(&mut buf)?;
        self.cs.set_high()?;

        ok(*res)
    }
    pub fn reset(&mut self) -> Result<(), SpiE>
    where
        CsE: core::fmt::Debug,
    {
        // '''
        // Resets the memory module
        // Resets any operations in progress or going on currently
        // puts the module in an idle state

        // Can be called when any corrupt tasks have been sent or
        // we need to kill commands that have been sent for it to do
        // '''
        self.cs.set_low()?;
        self.spi.write(&[CMD_RESET])?;
        self.cs.set_high()?;

        cortex_m::asm::delay(64_0000);
        Ok(())
    }

    pub fn free(self) -> (SPI, CS) {
        // '''
        // This function acts like a destructor basically freeing the SPI bus and the CS pin to be used
        // for any other purpose if needed.

        // It basically kills the instance we created of the memory module.

        // Might not be neeeded.

        // '''
        (self.spi, self.cs)
    }
