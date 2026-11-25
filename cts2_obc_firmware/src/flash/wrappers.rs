use super::layout::{FLASH_BASE_ADDRESS, PAGE_SIZE, SEGMENTS};
use crate::PERIPHERAL_FLASH;
use cortex_m::interrupt::free as critical_section;
use cts2_obc_logic::flash::{check_erase, check_read, check_write};
use cts2_obc_telecommands::error::FlashError;
use stm32l4xx_hal::flash::{FlashPage, WriteErase};

pub fn read(start: u32, buffer: &mut [u8]) -> Result<(), FlashError> {
    check_read(SEGMENTS, start, buffer.len())?;

    for (offset, byte) in buffer.iter_mut().enumerate() {
        *byte = unsafe { core::ptr::read_volatile((start as *const u8).wrapping_add(offset)) };
    }

    Ok(())
}

pub fn write(start: u32, data: &[u8]) -> Result<(), FlashError> {
    check_write(SEGMENTS, start, data.len())?;

    critical_section(|cs| {
        let mut flash = PERIPHERAL_FLASH.borrow(cs).borrow_mut();
        let flash = flash.as_mut().ok_or(FlashError::NotInitialized)?;

        let mut programming = flash
            .keyr
            .unlock_flash(&mut flash.sr, &mut flash.cr)
            .map_err(|_| FlashError::UnlockFailed)?;
        let flash_registers = unsafe { &*stm32l4xx_hal::stm32::FLASH::ptr() };

        flash_registers.sr.modify(|_, w| w); // Just a trick clearing errors bit before write
        let result = programming.write(start as usize, data);
        flash_registers.cr.modify(|_, w| w.pg().clear_bit()); // Only clear PG bit

        result.map_err(|_| FlashError::WriteFailed)
    })
}

pub fn erase(start: u32, page_count: u32) -> Result<(), FlashError> {
    let first_page = check_erase(SEGMENTS, FLASH_BASE_ADDRESS, PAGE_SIZE, start, page_count)?;

    critical_section(|cs| {
        let mut flash = PERIPHERAL_FLASH.borrow(cs).borrow_mut();
        let flash = flash.as_mut().ok_or(FlashError::NotInitialized)?;

        let mut programming = flash
            .keyr
            .unlock_flash(&mut flash.sr, &mut flash.cr)
            .map_err(|_| FlashError::UnlockFailed)?;
        let flash_registers = unsafe { &*stm32l4xx_hal::stm32::FLASH::ptr() };

        flash_registers.sr.modify(|_, w| w);
        (first_page..first_page + page_count).try_for_each(|page| {
            programming
                .erase_page(FlashPage(page as usize))
                .map_err(|_| FlashError::EraseFailed)
        })
    })
}
