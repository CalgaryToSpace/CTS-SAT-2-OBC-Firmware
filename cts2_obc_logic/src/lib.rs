#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

use cts2_obc_telecommands::Telecommand; // Sample include; use later.


pub fn multiply_by_2(i: u32) -> u32 {
    i * 2
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(multiply_by_2(21), 42);
        assert_eq!(multiply_by_2(0), 0);
    }
}
