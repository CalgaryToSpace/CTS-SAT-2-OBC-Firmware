#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;


pub enum Telecommand {
    HelloWorld,
    Ping,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(42, 42);
    }
}
