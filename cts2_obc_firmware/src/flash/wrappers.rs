use super::layout::{FLASH_BASE_ADDRESS, PAGE_SIZE, SEGMENTS};
use crate::PERIPHERAL_FLASH;
use cortex_m::interrupt::free as critical_section;
use cts2_obc_logic::flash::is_range_allowed;
use cts2_obc_telecommands::error::FlashError;
use stm32l4xx_hal::flash::{FlashPage, WriteErase};

pub fn read(start: u32, buffer: &mut [u8]) -> Result<(), FlashError> {
    let length = u32::try_from(buffer.len()).map_err(|_| FlashError::InvalidRange)?;
    let end = length
        .checked_sub(1)
        .and_then(|offset| start.checked_add(offset))
        .ok_or(FlashError::InvalidRange)?;

    if !is_range_allowed(SEGMENTS, start, end, |segment| segment.readable) {
        return Err(FlashError::ReadNotAllowed);
    }

    for (offset, byte) in buffer.iter_mut().enumerate() {
        *byte = unsafe { core::ptr::read_volatile((start as *const u8).wrapping_add(offset)) };
    }

    Ok(())
}

pub fn write(start: u32, data: &[u8]) -> Result<(), FlashError> {
    if !start.is_multiple_of(8) {
        return Err(FlashError::UnalignedWriteAddress);
    }

    let length = u32::try_from(data.len()).map_err(|_| FlashError::InvalidRange)?;
    let padded_length = length
        .checked_next_multiple_of(8)
        .ok_or(FlashError::InvalidRange)?;
    let end = padded_length
        .checked_sub(1)
        .and_then(|offset| start.checked_add(offset))
        .ok_or(FlashError::InvalidRange)?;

    if !is_range_allowed(SEGMENTS, start, end, |segment| segment.writable) {
        return Err(FlashError::WriteNotAllowed);
    }

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
    if !start.is_multiple_of(PAGE_SIZE) {
        return Err(FlashError::UnalignedEraseAddress);
    }

    let end = page_count
        .checked_mul(PAGE_SIZE)
        .and_then(|length| length.checked_sub(1))
        .and_then(|offset| start.checked_add(offset))
        .ok_or(FlashError::InvalidRange)?;

    if !is_range_allowed(SEGMENTS, start, end, |segment| segment.erasable) {
        return Err(FlashError::EraseNotAllowed);
    }

    let first_page = start
        .checked_sub(FLASH_BASE_ADDRESS)
        .ok_or(FlashError::InvalidRange)?
        / PAGE_SIZE;

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
