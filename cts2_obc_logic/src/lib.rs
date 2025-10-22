#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

use cts2_obc_telecommands::Telecommand; // Sample include; use later.


pub fn return_42() -> u32 {
    42
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(return_42(), 42);
    }
}
