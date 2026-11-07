#[derive(Clone, Copy)]
pub struct Segment {
    pub start: u32,
    pub end: u32,
    pub readable: bool,
    pub writable: bool,
    pub erasable: bool,
}

pub const fn sort_segments<const N: usize>(mut segments: [Segment; N]) -> [Segment; N] {
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

pub const fn assert_valid_segments(segments: &[Segment]) {
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

// WARNING: This function assumes that the segments are sorted and non overlapping
// This could improve to O(log n) incase the memory map is large, but for now this is good enough.
pub fn is_range_allowed(
    segments: &[Segment],
    start: u32,
    end: u32,
    attribute: impl Fn(&Segment) -> bool,
) -> bool {
    if start > end {
        return false;
    }

    let mut next = start;
    for segment in segments {
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

#[cfg(test)]
mod tests {
    use super::*;

    const READ_ONLY: Segment = Segment {
        start: 10,
        end: 19,
        readable: true,
        writable: false,
        erasable: false,
    };

    mod sorting {
        use super::*;

        #[test]
        fn orders_by_start() {
            const SORTED: [Segment; 3] = sort_segments([
                Segment {
                    start: 30,
                    end: 39,
                    ..READ_ONLY
                },
                READ_ONLY,
                Segment {
                    start: 20,
                    end: 29,
                    ..READ_ONLY
                },
            ]);
            assert_eq!(
                SORTED.map(|s| (s.start, s.end)),
                [(10, 19), (20, 29), (30, 39)]
            );
        }

        #[test]
        fn accepts_empty_input() {
            assert!(sort_segments::<0>([]).is_empty());
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn accepts_empty_layout() {
            assert_valid_segments(&[]);
        }

        #[test]
        fn accepts_single_address() {
            assert_valid_segments(&[Segment {
                start: 0,
                end: 0,
                ..READ_ONLY
            }]);
        }

        #[test]
        fn accepts_adjacent_segments() {
            const _: () = assert_valid_segments(&[
                READ_ONLY,
                Segment {
                    start: 20,
                    end: 29,
                    ..READ_ONLY
                },
            ]);
        }

        #[test]
        #[should_panic(expected = "Flash segment has invalid bounds")]
        fn rejects_reversed_bounds() {
            assert_valid_segments(&[Segment {
                start: 20,
                end: 19,
                ..READ_ONLY
            }]);
        }

        #[test]
        #[should_panic(expected = "Flash segments are not sorted by start")]
        fn rejects_unsorted_segments() {
            assert_valid_segments(&[
                Segment {
                    start: 30,
                    end: 39,
                    ..READ_ONLY
                },
                READ_ONLY,
            ]);
        }

        #[test]
        #[should_panic(expected = "Flash segments overlap")]
        fn rejects_shared_endpoint() {
            assert_valid_segments(&[
                READ_ONLY,
                Segment {
                    start: 19,
                    end: 29,
                    ..READ_ONLY
                },
            ]);
        }
    }

    mod ranges {
        use super::*;

        #[test]
        fn accepts_inclusive_bounds() {
            assert!(is_range_allowed(&[READ_ONLY], 10, 19, |s| s.readable));
        }

        #[test]
        fn accepts_single_address() {
            assert!(is_range_allowed(&[READ_ONLY], 19, 19, |s| s.readable));
        }

        #[test]
        fn accepts_adjacent_segments() {
            let segments = [
                READ_ONLY,
                Segment {
                    start: 20,
                    end: 29,
                    ..READ_ONLY
                },
            ];
            assert!(is_range_allowed(&segments, 10, 29, |s| s.readable));
        }

        #[test]
        fn skips_segments_before_range() {
            let segments = [
                READ_ONLY,
                Segment {
                    start: 40,
                    end: 49,
                    ..READ_ONLY
                },
            ];
            assert!(is_range_allowed(&segments, 40, 49, |s| s.readable));
        }

        #[test]
        fn rejects_reversed_bounds() {
            assert!(!is_range_allowed(&[READ_ONLY], 10, 9, |s| s.readable));
        }

        #[test]
        fn rejects_empty_layout() {
            assert!(!is_range_allowed(&[], 10, 10, |s| s.readable));
        }

        #[test]
        fn rejects_start_before_segment() {
            assert!(!is_range_allowed(&[READ_ONLY], 9, 10, |s| s.readable));
        }

        #[test]
        fn rejects_end_after_layout() {
            assert!(!is_range_allowed(&[READ_ONLY], 10, 20, |s| s.readable));
        }

        #[test]
        fn rejects_start_after_layout() {
            assert!(!is_range_allowed(&[READ_ONLY], 20, 20, |s| s.readable));
        }

        #[test]
        fn rejects_gap_between_segments() {
            let segments = [
                READ_ONLY,
                Segment {
                    start: 30,
                    end: 39,
                    ..READ_ONLY
                },
            ];
            assert!(!is_range_allowed(&segments, 10, 39, |s| s.readable));
        }

        #[test]
        fn rejects_denied_segment_inside_range() {
            let segments = [
                READ_ONLY,
                Segment {
                    start: 20,
                    end: 29,
                    readable: false,
                    ..READ_ONLY
                },
                Segment {
                    start: 30,
                    end: 39,
                    ..READ_ONLY
                },
            ];
            assert!(!is_range_allowed(&segments, 10, 39, |s| s.readable));
        }

        #[test]
        fn rejects_read_when_only_write_allowed() {
            let segment = Segment {
                readable: false,
                writable: true,
                ..READ_ONLY
            };
            assert!(!is_range_allowed(&[segment], 10, 19, |s| s.readable));
        }

        #[test]
        fn accepts_write_permission() {
            let segment = Segment {
                readable: false,
                writable: true,
                ..READ_ONLY
            };
            assert!(is_range_allowed(&[segment], 10, 19, |s| s.writable));
        }

        #[test]
        fn rejects_write_permission() {
            assert!(!is_range_allowed(&[READ_ONLY], 10, 19, |s| s.writable));
        }

        #[test]
        fn accepts_erase_permission() {
            let segment = Segment {
                readable: false,
                erasable: true,
                ..READ_ONLY
            };
            assert!(is_range_allowed(&[segment], 10, 19, |s| s.erasable));
        }

        #[test]
        fn rejects_erase_permission() {
            assert!(!is_range_allowed(&[READ_ONLY], 10, 19, |s| s.erasable));
        }

        #[test]
        fn accepts_full_address_space_without_overflow() {
            let segments = [
                Segment {
                    start: 0,
                    end: u32::MAX - 1,
                    ..READ_ONLY
                },
                Segment {
                    start: u32::MAX,
                    end: u32::MAX,
                    ..READ_ONLY
                },
            ];
            assert!(is_range_allowed(&segments, 0, u32::MAX, |s| s.readable));
        }

        #[test]
        fn accepts_max_address() {
            let segment = Segment {
                start: u32::MAX,
                end: u32::MAX,
                ..READ_ONLY
            };
            assert!(is_range_allowed(&[segment], u32::MAX, u32::MAX, |s| s.readable));
        }
    }
}
