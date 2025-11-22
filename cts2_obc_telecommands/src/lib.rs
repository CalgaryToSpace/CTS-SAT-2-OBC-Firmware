#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

//Telecommand Enum
pub enum Telecommand {
    Ping,
    LedOn,
    LedOff,
}

pub fn parse_command(input: &str) -> Result<Telecommand, ()> {
    match input.trim() {
        "PING" => Ok(Telecommand::Ping),
        "LED ON" => Ok(Telecommand::LedOn),
        "LED OFF" => Ok(Telecommand::LedOff),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(42, 42);
    }
}
