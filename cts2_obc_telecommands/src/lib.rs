#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

#[derive(Debug)]
pub enum Telecommand {
    Ping,
    LedOn,
    LedOff,
}

// TODO: Replace with meaningful telecommands
#[allow(clippy::result_unit_err)] // TODO: Fix the () error type to be enum or string
pub fn parse_telecommand(input: &str) -> Result<Telecommand, ()> {
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

    #[test]
    fn test_parse_telecommand_valid() {
        assert!(matches!(parse_telecommand("PING"), Ok(Telecommand::Ping)));
        assert!(matches!(parse_telecommand(" PING "), Ok(Telecommand::Ping)));
        assert!(matches!(
            parse_telecommand("LED ON"),
            Ok(Telecommand::LedOn)
        ));
        assert!(matches!(
            parse_telecommand("LED OFF"),
            Ok(Telecommand::LedOff)
        ));
        assert!(matches!(
            parse_telecommand(" LED OFF"),
            Ok(Telecommand::LedOff)
        ));
        assert!(matches!(
            parse_telecommand("LED OFF "),
            Ok(Telecommand::LedOff)
        ));
    }

    #[test]
    fn test_parse_telecommand_invalid() {
        assert!(matches!(parse_telecommand("PINGS"), Err(())));
        assert!(matches!(parse_telecommand("PONGS"), Err(())));
        assert!(matches!(parse_telecommand(""), Err(())));
        assert!(matches!(parse_telecommand("LEDON"), Err(())));
        assert!(matches!(parse_telecommand("LEDOFF"), Err(())));
    }
}
