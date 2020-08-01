//! Configuring the WatchDog Timer.
//! The watchdog timer settings are in a write-once register, and can
//! cannot be changed again until after a reboot.  Consequently, all
//! write-once settings are required to be set up-front at initialization
//! time.
use crate::hal;
use crate::target_device::WDT;

const SLOW_CLK_FREQ: u16 = 32768;
const WDT_CLK_DIVIDER: u16 = 128;

// wdt timeout in seconds * (SLOW_CLK_FREQ / WDT_CLK_DIVIDER) => timer initialization value
// maximum timeout of 15.996s is achieved using a counter value of 4095 or 0xFFF
fn wdt_ms_to_counter(ms: u16) -> u16 {
    (ms * (SLOW_CLK_FREQ / WDT_CLK_DIVIDER)) / 1000
}

/// Errors resulting from watchdog operations
pub enum Error {
    /// Attempt to change watchdog mode failed because once set, the watchdog
    /// mode may no longer be changed.
    WatchdogModeImmutable,
}
/// Configuration settings when enabling the watchdog timer
pub struct WdtBuilder {
    /// The watchdog timer peripheral object
    pub wdt: WDT,
    /// The 12-bit down-counter used by the watchdog timer decrements once
    /// every SLOW_CLK/128
    pub counter: u16,
    /// If enabled, a fault will be issued if the watchdog is reset while
    /// the down-counter is still above this value
    /// A value of zero will be converted into the counter value when applying
    /// the values to the watchdog peripheral (no fault).
    pub delta: u16,
    /// Whether watchdog faults should raise an interrupt
    pub fault_int: bool,
    /// Whether watchdog faults should issue a reset
    pub fault_res: bool,
    /// Whether watchdog faults should only reset the processor
    pub fault_res_cpu_only: bool,
    /// Whether the watchdog should continue counting during debug states
    pub debug_halt: bool,
    /// Whether the watchdog should continue counting during idle states
    pub idle_halt: bool,
    /// Whether the watchdog timer is disabled
    pub disabled: bool,
}

impl core::ops::Deref for WdtBuilder {
    type Target = WDT;

    fn deref(&self) -> &Self::Target {
        &self.wdt
    }
}

impl core::ops::DerefMut for WdtBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wdt
    }
}

impl From<WDT> for WdtBuilder {
    fn from(wdt: WDT) -> Self {
        let counter = wdt.mr.read().wdv().bits();
        let delta = wdt.mr.read().wdd().bits();
        let fault_int = wdt.mr.read().wdfien().bits();
        let fault_res = wdt.mr.read().wdrsten().bits();
        let fault_res_cpu_only = wdt.mr.read().wdrproc().bits();
        let debug_halt = wdt.mr.read().wddbghlt().bits();
        let idle_halt = wdt.mr.read().wdidlehlt().bits();
        let disabled = wdt.mr.read().wddis().bits();
        Self {
            wdt,
            counter,
            delta,
            fault_int,
            fault_res,
            fault_res_cpu_only,
            debug_halt,
            idle_halt,
            disabled,
        }
    }
}

impl WdtBuilder {
    /// Instantiate a Watchdog builder, for configuring a watchdog timer
    /// peripheral.
    pub fn new(wdt: WDT) -> Self {
        WdtBuilder::from(wdt)
    }

    /// Initialize the counter value for the watchdog timer to count down from.
    pub fn counter(mut self, counter: u16) -> Self {
        self.counter = counter;
        self
    }

    /// Initialize the counter value for the watchdog timer to count down from
    /// based on how many milliseconds should pass before it expires.
    pub fn counter_from_ms(mut self, ms: u16) -> Self {
        self.counter = wdt_ms_to_counter(ms);
        self
    }

    /// Configure the watchdog to fault if the current counter value is greater
    /// than the configured delta.  If set to 0, then the delta will instead be
    /// programmed with the requested counter value, effectively disabling it.
    pub fn delta(mut self, delta: u16) -> Self {
        self.delta = delta;
        self
    }

    /// Configure the watchdog to not fault if refreshed too soon.
    pub fn no_delta(mut self) -> Self {
        self.delta = 0;
        self
    }

    /// Configure the watchdog to raise an interrupt when it faults.
    pub fn interrupt_on_fault(mut self) -> Self {
        self.fault_int = true;
        self
    }

    /// Configure the watchdog to not raise an interrupt when it faults.
    pub fn no_interrupt_on_fault(mut self) -> Self {
        self.fault_int = true;
        self
    }

    /// Configure the watchdog to reset the processor and all peripherals when
    /// resetting from a fault.
    pub fn full_reset_on_fault(mut self) -> Self {
        self.fault_res = true;
        self.fault_res_cpu_only = false;
        self
    }

    /// Configure the watchdog to reset only the processor when resetting from
    /// a fault.
    pub fn cpu_reset_on_fault(mut self) -> Self {
        self.fault_res = true;
        self.fault_res_cpu_only = true;
        self
    }

    /// Configure the watchdog to not reset when faulting (generally in this
    /// case the watchdog timer should also be configured to
    /// `interrupt_on_fault()`, or instead be `disabled()`).
    pub fn no_reset_on_fault(mut self) -> Self {
        self.fault_res = false;
        self
    }

    /// Suspend the watchdog countdown when the processor is in debug mode.
    pub fn debug_halt(mut self) -> Self {
        self.debug_halt = true;
        self
    }

    /// The watchdog countdown should continue when the processor is in
    /// debug mode.
    pub fn no_debug_halt(mut self) -> Self {
        self.debug_halt = false;
        self
    }

    /// Suspend the watchdog countdown when the processor is idle.
    pub fn idle_halt(mut self) -> Self {
        self.idle_halt = true;
        self
    }

    /// The watchdog countdown should continue when the processor is idle.
    pub fn no_idle_halt(mut self) -> Self {
        self.idle_halt = false;
        self
    }

    /// Build the watchdog timer in a disabled state. No further configuration
    /// of this peripheral is possible because the configuration register for
    /// this peripheral is write-once.
    pub fn disable(self) -> WatchdogDisabled {
        // TODO: Verify that the write operation reads back with the same
        // value.  If the register was already written, the write will fail
        // and we should error.
        self.mr.write(|w| w.wddis().set_bit());
        WatchdogDisabled { _wdt: self.wdt }
    }

    /// Build the watchdog timer with the provided configuration values. No
    /// further configuration of this peripheral is possible because the
    /// configuration register for this peripheral is write-once.
    pub fn build(mut self) -> Watchdog {
        if self.delta == 0 {
            self.delta = self.counter;
        }
        // TODO: Verify that the write operation reads back with the same
        // value.  If the register was already written, the write will fail
        // and we should error.
        self.mr.write(|w| unsafe {
            w.wdv()
                .bits(self.counter)
                .wdfien()
                .bit(self.fault_int)
                .wdrsten()
                .bit(self.fault_res)
                .wdrproc()
                .bit(self.fault_res_cpu_only)
                .wdd()
                .bits(self.delta)
                .wddbghlt()
                .bit(self.debug_halt)
                .wdidlehlt()
                .bit(self.idle_halt)
                .wddis()
                .clear_bit()
        });
        Watchdog { wdt: self.wdt }
    }
}

/// This is only useful to feed the watchdog in its default state, before
/// applying the desired configuration.
impl hal::watchdog::Watchdog for WdtBuilder {
    type Error = core::convert::Infallible;
    /// Feeds an existing watchdog to ensure the processor isn't reset.
    /// Sometimes commonly referred to as "kicking" or "refreshing".
    fn try_feed(&mut self) -> Result<(), Self::Error> {
        self.cr
            .write_with_zero(|w| w.key().passwd().wdrstt().set_bit());
        Ok(())
    }
}

impl hal::watchdog::Enable for WdtBuilder {
    type Error = core::convert::Infallible;
    type Target = Watchdog;
    type Time = u8;

    /// Starts the watchdog with a given period, typically once this is done
    /// the watchdog needs to be `feed()` periodically, or the processor would be
    /// reset.
    ///
    /// This consumes the value and returns the `Watchdog` object that you must
    /// `feed()`.
    fn try_start<T>(self, period: T) -> Result<Self::Target, Self::Error>
    where
        T: Into<Self::Time>,
    {
        let period_ms: u16 = (period.into() as u16) * 1000;
        Ok(self.counter_from_ms(period_ms).build())
    }
}

impl hal::watchdog::Disable for WdtBuilder {
    type Error = core::convert::Infallible;
    type Target = WatchdogDisabled;
    /// Disables a running watchdog timer so the processor won't be reset.
    ///
    /// This consumes the value and returns the `WatchdogDisabled` object that
    /// has no useful operations.
    fn try_disable(self) -> Result<Self::Target, Self::Error> {
        Ok(self.disable())
    }
}

/// Represents a Watchdog timer that has been configured to be disabled.
/// Because the configuration register is write-once, no further configuration
/// is possible.
pub struct WatchdogDisabled {
    _wdt: WDT,
}

impl From<WDT> for WatchdogDisabled {
    fn from(wdt: WDT) -> Self {
        WdtBuilder::from(wdt).disable()
    }
}

impl hal::watchdog::Enable for WatchdogDisabled {
    type Error = Error;
    type Target = Watchdog;
    type Time = u8;

    /// The watchdog, once disabled, may not be enabled without resetting the
    /// whole system.
    ///
    /// This consumes the value and returns an Error object.
    fn try_start<T>(self, _period: T) -> Result<Self::Target, Self::Error>
    where
        T: Into<Self::Time>,
    {
        Err(Error::WatchdogModeImmutable)
    }
}

/// The state object for the watchdog timer peripheral on the SoC.
pub struct Watchdog {
    wdt: WDT,
}

impl core::ops::Deref for Watchdog {
    type Target = WDT;

    fn deref(&self) -> &Self::Target {
        &self.wdt
    }
}

impl core::ops::DerefMut for Watchdog {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wdt
    }
}

impl hal::watchdog::Watchdog for Watchdog {
    type Error = core::convert::Infallible;
    /// Feeds an existing watchdog to ensure the processor isn't reset.
    /// Sometimes commonly referred to as "kicking" or "refreshing".
    fn try_feed(&mut self) -> Result<(), Self::Error> {
        // TODO: Verify that the watchdog is in fact enabled, or fail the feeding.
        self.cr
            .write_with_zero(|w| w.key().passwd().wdrstt().set_bit());
        Ok(())
    }
}
