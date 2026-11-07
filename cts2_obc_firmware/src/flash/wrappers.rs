use super::layout::SEGMENTS;
use cts2_obc_logic::flash::is_range_allowed;
use cts2_obc_telecommands::error::FlashError;

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
