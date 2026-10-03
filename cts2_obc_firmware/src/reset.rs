use stm32l4xx_hal::pac::rcc::csr;

pub enum ResetCause {
    Unknown,
    LowPower,
    WindowWatchdog,
    IndependentWatchdog,
    Software,
    ExternalPin,
    BrownOut,
    OptionalByteLoader,
    Firewall,
}

impl ResetCause {
    pub fn from_csr(csr: csr::R) -> Self {
        if csr.lpwrstf().bit_is_set() {
            Self::LowPower
        } else if csr.wwdgrstf().bit_is_set() {
            Self::WindowWatchdog
        } else if csr.iwdgrstf().bit_is_set() {
            Self::IndependentWatchdog
        } else if csr.sftrstf().bit_is_set() {
            Self::Software
        } else if csr.pinrstf().bit_is_set() {
            Self::ExternalPin
        } else if csr.borrstf().bit_is_set() {
            Self::BrownOut
        } else if csr.oblrstf().bit_is_set() {
            Self::OptionalByteLoader
        } else if csr.fwrstf().bit_is_set() {
            Self::Firewall
        } else {
            Self::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ResetCause::Unknown => "Unknown",
            ResetCause::LowPower => "Low Power",
            ResetCause::WindowWatchdog => "Window Watchdog",
            ResetCause::IndependentWatchdog => "Independent Watchdog",
            ResetCause::Software => "Software",
            ResetCause::ExternalPin => "External Pin",
            ResetCause::BrownOut => "Brown Out",
            ResetCause::OptionalByteLoader => "Optional Byte Loader",
            ResetCause::Firewall => "Firewall",
        }
    }
}
