#[derive(Clone, Copy)]
pub struct Segment {
    pub start: u32,
    pub end: u32,
    pub readable: bool,
    pub writable: bool,
    pub erasable: bool,
}

const fn sort_segments<const N: usize>(mut segments: [Segment; N]) -> [Segment; N] {
    let mut i = 1;
    while i < N {
        let segment = segments[i];
        let mut j = i;
        while j > 0 && segments[j - 1].start > segment.start {
            segments[j] = segments[j - 1];
            j -= 1;
        }
        segments[j] = segment;
        i += 1;
    }
    segments
}

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


const fn assert_valid_segments(segments: &[Segment]) {
    let mut i = 0;
    while i < segments.len() {
        let segment = &segments[i];
        assert!(
            segment.start <= segment.end,
            "Flash segment has invalid bounds"
        );

        if i > 0 {
            let previous = &segments[i - 1];
            assert!(
                previous.start <= segment.start,
                "Flash segments are not sorted by start"
            );
            assert!(previous.end < segment.start, "Flash segments overlap");
        }
        i += 1;
    }
}

const _: () = assert_valid_segments(SEGMENTS);


// WARNING: This function assumes that the segments are sorted and non overlapping
// This could improve to O(log n) incase the memory map is large, but for now this is good enough.
pub fn is_range_allowed(start: u32, end: u32, attribute: impl Fn(&Segment) -> bool) -> bool {
    if start > end {
        return false;
    }

    let mut next = start;
    for segment in SEGMENTS {
        if segment.end < next {
            continue;
        }
        if segment.start > next || !attribute(segment) {
            return false;
        }
        if segment.end >= end {
            return true;
        }
        next = segment.end + 1;
    }

    false
}
