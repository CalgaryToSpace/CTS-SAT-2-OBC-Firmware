pub struct Segment {
    pub start: u32,
    pub end: u32,
    pub readable: bool,
    pub writable: bool,
    pub erasable: bool,
}

pub const SEGMENTS: &[Segment] = &[
    // Demo read-only segment
    Segment {
        start: 0x0800_0000,
        end: 0x0800_3FFF,
        readable: true,
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

    // Demo dead segment
    Segment {
        start: 0x080F_0000,
        end: 0x080F_BFFF,
        readable: false,
        writable: false,
        erasable: false,
    },
];
