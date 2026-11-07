use cts2_obc_logic::flash::{Segment, assert_valid_segments, sort_segments};


// Undeclared segments will be treated as dead segments, which are not readable, writable, or
// erasable. If we want to change behavior as full permissions, check logic crate, flash.rs,
// is_range_allowed function.
// TODO: Replace the demo segments with the actual flash memory map of the OBC.
pub const SEGMENTS: &[Segment] = &sort_segments([
    // Demo read-only segment
    Segment {
        start: 0x0800_0000,
        end: 0x0800_3FFF,
        readable: true,
        writable: false,
        erasable: false,
    },
    // Demo dead segment
    Segment {
        start: 0x080F_0000,
        end: 0x080F_BFFF,
        readable: false,
        writable: false,
        erasable: false,
    },
    // Demo read-write segment
    Segment {
        start: 0x080F_C000,
        end: 0x080F_DFFF,
        readable: true,
        writable: true,
        erasable: false,
    },
    // Demo read-erase segment
    Segment {
        start: 0x080F_E000,
        end: 0x080F_FFFF,
        readable: true,
        writable: false,
        erasable: true,
    },
]);

const _: () = assert_valid_segments(SEGMENTS);
