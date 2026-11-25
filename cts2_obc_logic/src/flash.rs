use cts2_obc_telecommands::error::FlashError;

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
// Which is checked during build time already
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

fn end_address(start: u32, length: u32) -> Result<u32, FlashError> {
    length
        .checked_sub(1)
        .and_then(|offset| start.checked_add(offset))
        .ok_or(FlashError::InvalidRange)
}

pub fn check_read(segments: &[Segment], start: u32, length: usize) -> Result<(), FlashError> {
    let length = u32::try_from(length).map_err(|_| FlashError::InvalidRange)?;
    let end = end_address(start, length)?;

    if !is_range_allowed(segments, start, end, |segment| segment.readable) {
        return Err(FlashError::ReadNotAllowed);
    }

    Ok(())
}

pub fn check_write(segments: &[Segment], start: u32, length: usize) -> Result<(), FlashError> {
    if !start.is_multiple_of(8) {
        return Err(FlashError::UnalignedWriteAddress);
    }

    let length = u32::try_from(length).map_err(|_| FlashError::InvalidRange)?;
    let padded_length = length
        .checked_next_multiple_of(8)
        .ok_or(FlashError::InvalidRange)?;
    let end = end_address(start, padded_length)?;

    if !is_range_allowed(segments, start, end, |segment| segment.writable) {
        return Err(FlashError::WriteNotAllowed);
    }

    Ok(())
}

pub fn check_erase(
    segments: &[Segment],
    base_address: u32,
    page_size: u32,
    start: u32,
    page_count: u32,
) -> Result<u32, FlashError> {
    if !start.is_multiple_of(page_size) {
        return Err(FlashError::UnalignedEraseAddress);
    }

    let length = page_count
        .checked_mul(page_size)
        .ok_or(FlashError::InvalidRange)?;
    let end = end_address(start, length)?;

    if !is_range_allowed(segments, start, end, |segment| segment.erasable) {
        return Err(FlashError::EraseNotAllowed);
    }

    let first_page = start
        .checked_sub(base_address)
        .ok_or(FlashError::InvalidRange)?
        / page_size;

    Ok(first_page)
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

    mod end_address {
        use super::*;

        #[test]
        fn includes_last_byte() {
            assert_eq!(end_address(10, 10), Ok(19));
        }

        #[test]
        fn reaches_max_address() {
            assert_eq!(end_address(u32::MAX, 1), Ok(u32::MAX));
        }

        #[test]
        fn rejects_overflow() {
            assert_eq!(end_address(u32::MAX, 2), Err(FlashError::InvalidRange));
        }

        #[test]
        fn rejects_zero_length() {
            assert_eq!(end_address(10, 0), Err(FlashError::InvalidRange));
        }
    }

    mod read {
        use super::*;

        #[test]
        fn requires_readable_permission() {
            let unreadable = Segment {
                readable: false,
                writable: true,
                erasable: true,
                ..READ_ONLY
            };
            assert_eq!(check_read(&[READ_ONLY], 10, 10), Ok(()));
            assert_eq!(
                check_read(&[unreadable], 10, 10),
                Err(FlashError::ReadNotAllowed)
            );
        }

        #[test]
        fn rejects_length_beyond_u32() {
            assert_eq!(
                check_read(&[READ_ONLY], 10, usize::MAX),
                Err(FlashError::InvalidRange)
            );
        }
    }

    mod write {
        use super::*;

        const WRITABLE: Segment = Segment {
            start: 16,
            end: 23,
            writable: true,
            ..READ_ONLY
        };

        #[test]
        fn requires_writable_permission() {
            let read_only = Segment {
                writable: false,
                ..WRITABLE
            };
            assert_eq!(check_write(&[WRITABLE], 16, 8), Ok(()));
            assert_eq!(
                check_write(&[read_only], 16, 8),
                Err(FlashError::WriteNotAllowed)
            );
        }

        #[test]
        fn rejects_unaligned_start() {
            assert_eq!(
                check_write(&[WRITABLE], 17, 1),
                Err(FlashError::UnalignedWriteAddress)
            );
        }

        #[test]
        fn includes_padding_in_range() {
            let short = Segment {
                end: 19,
                ..WRITABLE
            };
            assert_eq!(
                check_write(&[short], 16, 1),
                Err(FlashError::WriteNotAllowed)
            );
        }

        #[test]
        fn rejects_padding_overflow() {
            assert_eq!(
                check_write(&[WRITABLE], 16, u32::MAX as usize),
                Err(FlashError::InvalidRange)
            );
        }
    }

    mod erase {
        use super::*;

        const ERASABLE: Segment = Segment {
            erasable: true,
            ..READ_ONLY
        };

        #[test]
        fn requires_erasable_permission() {
            assert_eq!(check_erase(&[ERASABLE], 0, 10, 10, 1), Ok(1));
            assert_eq!(
                check_erase(&[READ_ONLY], 0, 10, 10, 1),
                Err(FlashError::EraseNotAllowed)
            );
        }

        #[test]
        fn first_page_is_relative_to_base() {
            assert_eq!(check_erase(&[ERASABLE], 10, 10, 10, 1), Ok(0));
        }

        #[test]
        fn covers_whole_page() {
            let short = Segment {
                end: 18,
                ..ERASABLE
            };
            assert_eq!(
                check_erase(&[short], 0, 10, 10, 1),
                Err(FlashError::EraseNotAllowed)
            );
        }

        #[test]
        fn rejects_unaligned_start() {
            assert_eq!(
                check_erase(&[ERASABLE], 0, 10, 15, 1),
                Err(FlashError::UnalignedEraseAddress)
            );
        }

        #[test]
        fn rejects_page_count_overflow() {
            assert_eq!(
                check_erase(&[ERASABLE], 0, 10, 10, u32::MAX),
                Err(FlashError::InvalidRange)
            );
        }

        #[test]
        fn rejects_start_below_base() {
            assert_eq!(
                check_erase(&[ERASABLE], 20, 10, 10, 1),
                Err(FlashError::InvalidRange)
            );
        }
    }
}
