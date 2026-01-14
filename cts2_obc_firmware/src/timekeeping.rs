use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::interrupt::free as critical_section;

/// True after successful init.
static INIT_DONE: AtomicBool = AtomicBool::new(false);

/// Accumulated cycles (u64) to handle DWT 32-bit wrap-arounds.
static mut ACCUM_CYCLES: u64 = 0;

/// Last seen 32-bit cycle counter value.
static mut LAST_CYCCNT: u32 = 0;

/// Number of CPU cycles per millisecond (core_hz / 1000).
/// CPU core clock in Hz (set at init).
static mut CORE_HZ: u32 = 0;

/// Initialize the DWT cycle counter. Call once during startup.
/// `core_hz` is the CPU core clock frequency in Hz (e.g. 64_000_000).
pub fn init(core_hz: u32) -> Result<(), &'static str> {
    if core_hz < 1000 {
        return Err("core_hz too small");
    }

    // store core_hz for conversions
    let _cycles_per_ms = core_hz / 1000; // keep for a quick sanity derivation if needed

    unsafe {
        // Enable trace in DCB (set TRCENA in DEMCR at 0xE000EDFC bit 24)
        let demcr_ptr = 0xE000_EDFC as *mut u32;
        let demcr = core::ptr::read_volatile(demcr_ptr);
        core::ptr::write_volatile(demcr_ptr, demcr | (1 << 24));

        // Reset the cycle counter
        let cyccnt_ptr = (0xE000_1000u32 + 0x04) as *mut u32;
        core::ptr::write_volatile(cyccnt_ptr, 0u32);

        // Enable cycle counter (DWT CTRL bit 0)
        let dwt_ctrl_ptr = 0xE000_1000 as *mut u32;
        let ctrl = core::ptr::read_volatile(dwt_ctrl_ptr);
        core::ptr::write_volatile(dwt_ctrl_ptr, ctrl | 1u32);
    }

    unsafe {
        CORE_HZ = core_hz;
        LAST_CYCCNT = 0;
    }
    unsafe {
        ACCUM_CYCLES = 0;
    }
    INIT_DONE.store(true, Ordering::Release);

    Ok(())
}

/// Returns uptime in milliseconds since `init` was called.
/// If `init` hasn't been called successfully, returns 0.
pub fn uptime_ms() -> u64 {
    if !INIT_DONE.load(Ordering::Acquire) {
        return 0;
    }

    // Read current CYCCNT (32-bit) from DWT->CYCCNT at 0xE0001004
    let now_lo: u32 = unsafe { core::ptr::read_volatile((0xE000_1000u32 + 0x04) as *const u32) };

    // Update accumulated cycles handling wrap-around inside a critical section
    critical_section(|_| {
        let last = unsafe { LAST_CYCCNT };
        let delta = now_lo.wrapping_sub(last) as u64;
        if delta != 0 {
            unsafe {
                ACCUM_CYCLES = ACCUM_CYCLES.wrapping_add(delta);
            }
            unsafe {
                LAST_CYCCNT = now_lo;
            }
        }
    });

    let total_cycles = critical_section(|_| unsafe { ACCUM_CYCLES });

    // Use 128-bit math to avoid overflow and apply rounding when converting to ms.
    let core_hz = critical_section(|_| unsafe { CORE_HZ as u128 });
    if core_hz == 0 {
        return 0;
    }
    let cycles128 = total_cycles as u128;
    let ms = (cycles128 * 1000u128 + core_hz / 2u128) / core_hz;
    ms as u64
}
