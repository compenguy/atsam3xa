//! Delays

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::hal::blocking::delay::{DelayMs, DelayUs};
use crate::time::Hertz;

/// Timer object for requesting blocking delays, much like sleep().
pub struct Delay<PERIPH> {
    p: PERIPH,
    refclock: Hertz,
}

/// System timer (SysTick) as a delay provider
impl Delay<SYST> {
    /// Configures the system timer (SysTick) as a delay provider
    pub fn new(mut syst: SYST, core_speed: Hertz) -> Self {
        syst.set_clock_source(SystClkSource::Core);

        Delay {
            p: syst,
            refclock: core_speed,
        }
    }

    /// Releases the system timer (SysTick) resource
    pub fn free(self) -> SYST {
        self.p
    }
}

impl DelayMs<u32> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        self.try_delay_us(ms * 1_000)
    }
}

impl DelayMs<u16> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_ms(&mut self, ms: u16) -> Result<(), Self::Error> {
        self.try_delay_ms(ms as u32)
    }
}

impl DelayMs<u8> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_ms(&mut self, ms: u8) -> Result<(), Self::Error> {
        self.try_delay_ms(ms as u32)
    }
}

impl DelayUs<u32> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        const MAX_RVR: u32 = 0x00FF_FFFF;

        let mut total_rvr = us * (self.refclock.0 / 1_000_000);

        while total_rvr != 0 {
            let current_rvr = if total_rvr <= MAX_RVR {
                total_rvr
            } else {
                MAX_RVR
            };

            self.p.set_reload(current_rvr);
            self.p.clear_current();
            self.p.enable_counter();

            // Update the tracking variable while we are waiting...
            total_rvr -= current_rvr;

            while !self.p.has_wrapped() {}

            self.p.disable_counter();
        }
        Ok(())
    }
}

impl DelayUs<u16> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_us(&mut self, us: u16) -> Result<(), Self::Error> {
        self.try_delay_us(us as u32)
    }
}

impl DelayUs<u8> for Delay<SYST> {
    type Error = core::convert::Infallible;
    fn try_delay_us(&mut self, us: u8) -> Result<(), Self::Error> {
        self.try_delay_us(us as u32)
    }
}
