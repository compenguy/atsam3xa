//! Configuring the system clock sources.
//! You will typically need to create an instance of `SystemClocks`
//! before you can set up most of the peripherals on the atsam3x target.
//! The other types in this module are used to enforce at compile time
//! that the peripherals have been correctly configured.
use crate::target_device;
use crate::time::{Hertz, MegaHertz};
use crate::PeripheralID;
use target_device::generic::Variant;
use target_device::pmc::ckgr_mor::MOSCRCF_A::*;
use target_device::pmc::pmc_mckr::{CSS_A::*, PRES_A::*};
use target_device::supc::sr::OSCSEL_A::*;
use target_device::{PMC, SUPC};

/// Valid frequency settings for the Fast RC oscillator
pub type FastRCFreq = target_device::pmc::ckgr_mor::MOSCRCF_A;
/// Valid clock sources for the system master clock
pub type ClockSource = target_device::pmc::pmc_mckr::CSS_A;
/// Valid prescaler values for the system master clock
pub type ClockPrescaler = target_device::pmc::pmc_mckr::PRES_A;

/// Oscillator sources that can be used by the slow clock.
///
/// The Low Power RC oscillator starts up faster, but is less accurate, so it's
/// the default clock for the system.  If more accurate timing is required,
/// switch to the Low Power Crystal oscillator.  Once the LP Crystal osc. has
/// been enabled, it is not possible to switch back.
pub enum SlowClockSource {
    /// Slow clock RC oscillator, runs at 32000Hz and less accurate
    LowPowerRC,
    /// Slow clock external crystal oscillator, runs at 32768Hz and
    /// more accurate
    LowPowerXtal32Khz,
}

/// Oscillator sources that can be used by the main clock.
///
/// The FastRc clock is configurable across three frequencies
/// - 4 MHz (uncalibrated, system power-on default)
/// - 8 MHz (calibrated)
/// - 12 MHz (calibrated)
///
/// The FastRc clock starts up quickly, but generally has lower accuracy than
/// whatever external crystal or ceramic oscillator has been connected, in
/// spite of the calibration on the 8MHz and 12MHz frequencies.
///
/// The Main Clock Crystal frequency is determined by the board designer, but
/// 12MHz is a common value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainClockSource {
    /// Internal, RC oscillator
    FastRc(FastRCFreq),
    /// External Crystal or Ceramic oscillator
    MainXtal,
}

/// Divider to apply to the master clock when using either PLLA or UPLL as
/// the source.
pub enum PllDiv {
    /// No divider
    One = 0,
    /// Run the master clock at half the frequency of the source PLL
    Two = 1,
}

/// Configuration options for setting up the PLLA clock source.  The output
/// frequency is the source clock frequency * (mula + 1)/diva.  The clock is
/// disabled when mula = 0.
pub struct PllAClockConfig {
    /// Clock multiplier minus one
    pub mula: u16,
    /// Clock divider
    pub diva: u8,
    /// how many slow clock ticks are required for PLLA to settle
    pub count: u8,
}

/// `SystemClocks` encapsulates the PMC and SUPC clock hardware.
/// It provides a type safe way to configure the system clocks.
/// Initializing the `SystemClocks` instance configures the system to run at
/// 84MHz by configuring Main clock to run at 12MHz, then setting PLLA to run
/// at 14x the Main clock, and then setting Master Clock to divide PLLA by 2.
pub struct SystemClocks {
    /// Power Management Controller
    pub pmc: PMC,
    /// Power Supply Controller
    pub supc: SUPC,
}

impl core::ops::Deref for SystemClocks {
    type Target = PMC;

    fn deref(&self) -> &Self::Target {
        &self.pmc
    }
}

impl core::ops::DerefMut for SystemClocks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pmc
    }
}

impl SystemClocks {
    /// Select the specified slow clock oscillator, and clock the system to run
    /// at that frequency.
    pub fn with_slow_clk(pmc: PMC, supc: SUPC, use_external_crystal: bool) -> Self {
        let mut clk = Self { pmc, supc };
        if use_external_crystal {
            clk.enable_slow_clock_xtal();
        }
        clk.set_master_clock_source_and_prescaler(ClockSource::SLOW_CLK, None, false);
        clk
    }

    /// Set the main clock source, and clock the system to run at that frequency.
    pub fn with_main_clk(pmc: PMC, supc: SUPC, source: MainClockSource) -> Self {
        let mut clk = Self { pmc, supc };
        clk.set_main_clock_source(source);
        clk.set_master_clock_source_and_prescaler(ClockSource::MAIN_CLK, None, false);

        // If USB feature is enabled and main clock is set to 12MHz also enable
        // the UPLL clock.
        #[cfg(feature = "usb")]
        if clk.get_main_clock_rate() == MegaHertz(12).into() {
            clk.enable_upll(0x10u8);
        }

        clk
    }

    /// Configure the main clock to run off the main oscillator crystal, then
    /// configure PLLA to run at 14x that, set the system to run at half the
    /// frequency of PLLA, and if the usb feature is enabled, enable UPLL as well.
    pub fn with_plla_clk(pmc: PMC, supc: SUPC) -> Self {
        let mut clk = Self::with_main_clk(pmc, supc, MainClockSource::MainXtal);

        clk.configure_plla(PllAClockConfig {
            mula: 14 - 1,
            diva: 1,
            count: 0x3f,
        });
        clk.set_master_clock_source_and_prescaler(ClockSource::PLLA_CLK, None, true);

        // If USB feature is enabled and main clock is set to 12MHz also enable
        // the UPLL clock.
        #[cfg(feature = "usb")]
        if clk.get_main_clock_rate() == MegaHertz(12).into() {
            clk.enable_upll(0x10u8);
        }

        clk
    }

    /// Set the main clock source, and clock the system to run at that frequency.
    /// TODO: Either we have a bug in how we configure PLLA, or we have a bug in
    /// how to derive SysTick intervals based on PLLA configuration, either way
    /// until that's fixed, we'll default to using main clk.
    pub fn new(pmc: PMC, supc: SUPC) -> Self {
        Self::with_main_clk(pmc, supc, MainClockSource::MainXtal)
    }

    /// Return the frequency that the main clock is operating at
    pub fn get_slow_clock_rate(&self) -> Hertz {
        match self.supc.sr.read().oscsel().variant() {
            RC => Hertz(32000),
            CRYST => Hertz(32768),
        }
    }

    /// Return the frequency that the main clock is operating at
    pub fn get_main_clock_rate(&self) -> Hertz {
        match self.get_main_clock_source() {
            MainClockSource::FastRc(_4_MHZ) => MegaHertz(4).into(),
            MainClockSource::FastRc(_8_MHZ) => MegaHertz(8).into(),
            MainClockSource::FastRc(_12_MHZ) => MegaHertz(12).into(),
            MainClockSource::MainXtal => MegaHertz(12).into(),
        }
    }

    /// Return the frequency that the main clock is operating at, based
    /// on the slow clock rate.
    pub fn get_main_clock_rate_calibrated(&self) -> Hertz {
        // Wait until mainf has been calibrated since the last change of the
        // main clock
        while !self.ckgr_mcfr.read().mainfrdy().bits() {}

        // mainf is how many times the main clock ticks during the count of 16
        // slow clock cycles
        let mainf = self.ckgr_mcfr.read().mainf().bits() as u32;
        let mainf_freq = (mainf * self.get_slow_clock_rate().0) / 16;
        Hertz(mainf_freq)
    }

    /// Return the frequency that the plla clock is operating at
    pub fn get_plla_clock_rate(&self) -> Hertz {
        // plla clock = mainck * (mula + 1)/diva
        let mut tmp_clk = self.get_main_clock_rate();
        tmp_clk.0 *= (self.ckgr_pllar.read().mula().bits() + 1) as u32;
        tmp_clk.0 /= self.ckgr_pllar.read().diva().bits() as u32;
        tmp_clk
    }

    /// Return the frequency that the upll clock is operating at
    pub fn get_upll_clock_rate(&self) -> Hertz {
        // upll clock = mainck * 40
        // but it's only valid if mainck == 12MHz
        let mut tmp_clk = self.get_main_clock_rate();
        tmp_clk.0 *= 40;
        tmp_clk
    }

    /// Return the frequency that the master clock is operating at
    pub fn get_syscore(&mut self) -> Hertz {
        /* Determine clock frequency according to clock register values */
        let mut clk_unscaled: Hertz = match self.pmc_mckr.read().css().variant() {
            SLOW_CLK => self.get_slow_clock_rate(),
            MAIN_CLK => self.get_main_clock_rate(),
            PLLA_CLK => self.get_plla_clock_rate(),
            UPLL_CLK => self.get_upll_clock_rate(),
        };
        // Apply pll-specific divider if set
        if self.pmc_mckr.read().css().variant() == PLLA_CLK {
            clk_unscaled.0 /= 1 << (self.pmc_mckr.read().plladiv2().bits() as u8);
        }
        if self.pmc_mckr.read().css().variant() == PLLA_CLK {
            clk_unscaled.0 /= 1 << (self.pmc_mckr.read().uplldiv2().bits() as u8);
        }
        // Apply prescaler
        match self.pmc_mckr.read().pres().variant() {
            CLK_3 => Hertz(clk_unscaled.0 / 3),
            x => Hertz(clk_unscaled.0 >> (x as u8)),
        }
    }

    /// Slow clock is always enabled, but is sourced from a low-accuracy RC
    /// oscillator.  This enables the more accurate crystal oscillator and
    /// switch to use that as the slow clock source.  Once the crystal
    /// oscillator has been enabled, the RC oscillator is disabled and cannot
    /// be re-enabled.
    pub fn enable_slow_clock_xtal(&mut self) {
        self.supc
            .cr
            .write_with_zero(|w| w.key().passwd().xtalsel().set_bit());
    }

    /// Disabling the main clock is usually only done to enter low power/idle
    /// states.  It may only be re-enabled by an interrupt or rebooting.
    pub fn disable_main_clock(&mut self) {
        self.ckgr_mor.modify(|_, w| {
            w.key()
                .passwd()
                .moscrcen()
                .clear_bit()
                .moscxten()
                .clear_bit()
        });
    }

    /// Select the oscillator source to use for the main clock
    pub fn set_main_clock_source(&mut self, source: MainClockSource) {
        // TODO: if master clock source is PLLA or UPLL, and the current
        // MainClockSource is FastRC, a special procedure is required:
        //   - save the current master clock source selection
        //   - switch master clock to main
        //   - make the change to the main clock
        //   - wait for the change to settle
        //   - save the pll state
        //   - disable the pll
        //   - restore the pll state
        //     (the count value cannot be recovered from current state, though)
        //   - wait for the pll to settle
        //   - restore the master clock source selection

        // Crystal oscillator startup time
        // startup cycles = 8 * startup_time / SLCK
        let startup_time: u8 = 8;

        // To ensure a smooth transition in case other clocks are running off
        // main clock, we enable both clocks, wait out the startup time,
        // switch to the desired clock, then disable the unused clock
        self.ckgr_mor.modify(|_, w| unsafe {
            w.key()
                .passwd()
                .moscrcen()
                .set_bit()
                .moscxten()
                .set_bit()
                .moscxtst()
                .bits(startup_time)
        });
        // Wait until RC startup time runs out
        while !self.pmc_sr.read().moscrcs().bits() {}
        // Wait until Xtal startup time runs out
        while !self.pmc_sr.read().moscxts().bits() {}

        match source {
            MainClockSource::FastRc(f) => {
                // Set RC osc frequency
                self.ckgr_mor
                    .modify(|_, w| w.key().passwd().moscrcf().variant(f));
                // Let RC osc stabilize at new frequency
                while !self.pmc_sr.read().moscrcs().bits() {}

                // Switch main clock to RC osc
                self.ckgr_mor
                    .modify(|_, w| w.key().passwd().moscsel().clear_bit());
                // Wait until oscillator selection reports ready
                // 0 = done, 1 = in progress
                while !self.pmc_sr.read().moscsels().bits() {}

                // Disable unused xtal oscillator
                self.ckgr_mor
                    .modify(|_, w| w.key().passwd().moscxten().clear_bit());
            }
            MainClockSource::MainXtal => {
                self.ckgr_mor
                    .modify(|_, w| w.key().passwd().moscsel().set_bit());
                // Wait until oscillator selection reports ready
                // 0 = done, 1 = in progress
                while !self.pmc_sr.read().moscsels().bits() {}

                // Disable unused RC oscillator
                self.ckgr_mor
                    .modify(|_, w| w.key().passwd().moscrcen().clear_bit());
            }
        }
    }

    /// Return the currently-active main clock source.
    pub fn get_main_clock_source(&self) -> MainClockSource {
        match self.ckgr_mor.read().moscsel().bits() {
            true => MainClockSource::MainXtal,
            false => match self.ckgr_mor.read().moscrcf().variant() {
                Variant::Val(s) => MainClockSource::FastRc(s),
                Variant::Res(_) => unreachable!(),
            },
        }
    }

    /// Disable PLLA by setting the clock multiplier to zero.
    pub fn disable_plla(&mut self) {
        self.configure_plla(PllAClockConfig {
            mula: 0,
            diva: 1,
            count: 0,
        });
    }

    /// PLLA is always "enabled" but defaults to a multiplier of zero,
    /// effectively disabling it.  The resulting clock speed is the
    /// main clock * (mula + 1)/diva.
    pub fn configure_plla(&mut self, config: PllAClockConfig) {
        self.ckgr_pllar.write(|w| unsafe {
            w.one()
                .set_bit()
                .mula()
                .bits(config.mula)
                .diva()
                .bits(config.diva)
                .pllacount()
                .bits(config.count)
        });

        // Wait until pll is locked
        // 0 = not locked, 1 = locked
        while !self.pmc_sr.read().locka().bits() {}
    }

    /// Enable the UTMI PLL, primarily used for clocking USB. The count is how
    /// many slow clock ticks * 8 to wait for UPLL to settle.
    pub fn enable_upll(&mut self, count: u8) {
        // early exit if it's already enabled
        if self.ckgr_uckr.read().upllen().bits() {
            return;
        }

        self.ckgr_uckr
            .modify(|_, w| unsafe { w.upllen().set_bit().upllcount().bits(count) });

        // Wait until pll is locked
        // 0 = not locked, 1 = locked
        while !self.pmc_sr.read().locku().bits() {}
    }

    /// Disable the UTMI PLL, disabling the USB bus and any clocks configured
    /// to use it as a source.
    pub fn disable_upll(&mut self) {
        self.ckgr_uckr.modify(|_, w| w.upllen().clear_bit());
    }

    /// Select which clock source the master clock should use, along with some
    /// options for dividing the source clock.
    pub fn set_master_clock_source_and_prescaler(
        &mut self,
        source: ClockSource,
        prescaler: Option<ClockPrescaler>,
        pll_div2: bool,
    ) {
        // For PLLs, prescaler should be applied before changing the clock source
        if source == ClockSource::PLLA_CLK || source == ClockSource::UPLL_CLK {
            if let Some(prescaler) = prescaler {
                self.pmc_mckr.modify(|_, w| w.pres().variant(prescaler));

                // Wait for the prescaler to latch
                // 0 = not ready, 1 = ready
                while !self.pmc_sr.read().mckrdy().bits() {}
            }
        }
        // For switching to PLL, we have to prime it by first setting main clock
        // and the pll divider before we switch to the PLL
        if source == ClockSource::PLLA_CLK {
            self.pmc_mckr
                .modify(|_, w| w.css().main_clk().plladiv2().bit(pll_div2));
            while !self.pmc_sr.read().mckrdy().bits() {}
        }
        if source == ClockSource::UPLL_CLK {
            self.pmc_mckr
                .modify(|_, w| w.css().main_clk().uplldiv2().bit(pll_div2));
            while !self.pmc_sr.read().mckrdy().bits() {}
        }

        // Switch to the desired clock
        match source {
            ClockSource::SLOW_CLK => self.pmc_mckr.modify(|_, w| w.css().slow_clk()),
            ClockSource::MAIN_CLK => self.pmc_mckr.modify(|_, w| w.css().main_clk()),
            ClockSource::PLLA_CLK => self
                .pmc_mckr
                .modify(|_, w| w.css().plla_clk().plladiv2().bit(pll_div2)),
            ClockSource::UPLL_CLK => self
                .pmc_mckr
                .modify(|_, w| w.css().upll_clk().uplldiv2().bit(pll_div2)),
        }

        // Wait until master clock reports ready
        // 0 = not ready, 1 = ready
        while !self.pmc_sr.read().mckrdy().bits() {}

        // For slow and main clocks, prescaler should be applied after changing
        // the clock source
        if source == ClockSource::SLOW_CLK || source == ClockSource::MAIN_CLK {
            if let Some(prescaler) = prescaler {
                self.pmc_mckr.modify(|_, w| w.pres().variant(prescaler));

                // Wait for the prescaler to latch
                // 0 = not ready, 1 = ready
                while !self.pmc_sr.read().mckrdy().bits() {}
            }
        }
    }

    /// Enable the clock for the specified peripheral.  Some peripherals'
    /// clocks are not under PMC control - passing the ID for these clocks
    /// will silently do nothing.
    pub fn enable_peripheral_clock(&mut self, pid: PeripheralID) {
        match pid {
            PeripheralID::PMC => (),  // Clock not under PMC control
            PeripheralID::EFC0 => (), // Clock not under PMC control
            PeripheralID::EFC1 => (), // Clock not under PMC control
            PeripheralID::UART => self.pmc_pcer0.write_with_zero(|w| w.pid8().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::SDRAMC => self.pmc_pcer0.write_with_zero(|w| w.pid10().set_bit()),
            PeripheralID::PIOA => self.pmc_pcer0.write_with_zero(|w| w.pid11().set_bit()),
            PeripheralID::PIOB => self.pmc_pcer0.write_with_zero(|w| w.pid12().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::PIOC => self.pmc_pcer0.write_with_zero(|w| w.pid13().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::PIOD => self.pmc_pcer0.write_with_zero(|w| w.pid14().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::PIOE => self.pmc_pcer0.write_with_zero(|w| w.pid15().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::PIOF => self.pmc_pcer0.write_with_zero(|w| w.pid16().set_bit()),
            PeripheralID::USART0 => self.pmc_pcer0.write_with_zero(|w| w.pid17().set_bit()),
            PeripheralID::USART1 => self.pmc_pcer0.write_with_zero(|w| w.pid18().set_bit()),
            PeripheralID::USART2 => self.pmc_pcer0.write_with_zero(|w| w.pid19().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::USART3 => self.pmc_pcer0.write_with_zero(|w| w.pid20().set_bit()),
            PeripheralID::HSMCI => self.pmc_pcer0.write_with_zero(|w| w.pid21().set_bit()),
            PeripheralID::TWI0 => self.pmc_pcer0.write_with_zero(|w| w.pid22().set_bit()),
            PeripheralID::TWI1 => self.pmc_pcer0.write_with_zero(|w| w.pid23().set_bit()),
            PeripheralID::SPI0 => self.pmc_pcer0.write_with_zero(|w| w.pid24().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::SPI1 => self.pmc_pcer0.write_with_zero(|w| w.pid25().set_bit()),
            PeripheralID::SSC => self.pmc_pcer0.write_with_zero(|w| w.pid26().set_bit()),
            PeripheralID::TC0 => self.pmc_pcer0.write_with_zero(|w| w.pid27().set_bit()),
            PeripheralID::TC1 => self.pmc_pcer0.write_with_zero(|w| w.pid28().set_bit()),
            PeripheralID::TC2 => self.pmc_pcer0.write_with_zero(|w| w.pid29().set_bit()),
            PeripheralID::TC3 => self.pmc_pcer0.write_with_zero(|w| w.pid30().set_bit()),
            PeripheralID::TC4 => self.pmc_pcer0.write_with_zero(|w| w.pid31().set_bit()),
            PeripheralID::TC5 => self.pmc_pcer1.write_with_zero(|w| w.pid32().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC6 => self.pmc_pcer1.write_with_zero(|w| w.pid33().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC7 => self.pmc_pcer1.write_with_zero(|w| w.pid34().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC8 => self.pmc_pcer1.write_with_zero(|w| w.pid35().set_bit()),
            PeripheralID::PWM => self.pmc_pcer1.write_with_zero(|w| w.pid36().set_bit()),
            PeripheralID::ADC => self.pmc_pcer1.write_with_zero(|w| w.pid37().set_bit()),
            PeripheralID::DACC => self.pmc_pcer1.write_with_zero(|w| w.pid38().set_bit()),
            PeripheralID::DMAC => self.pmc_pcer1.write_with_zero(|w| w.pid39().set_bit()),
            PeripheralID::UOTGHS => self.pmc_pcer1.write_with_zero(|w| w.pid40().set_bit()),
            PeripheralID::TRNG => self.pmc_pcer1.write_with_zero(|w| w.pid41().set_bit()),
            #[cfg(feature = "sam3x")]
            PeripheralID::EMAC => self.pmc_pcer1.write_with_zero(|w| w.pid42().set_bit()),
            PeripheralID::CAN0 => self.pmc_pcer1.write_with_zero(|w| w.pid43().set_bit()),
            PeripheralID::CAN1 => self.pmc_pcer1.write_with_zero(|w| w.pid44().set_bit()),
        }
    }

    /// Disable the clock for the specified peripheral.  Some peripherals'
    /// clocks are not under PMC control - passing the ID for these clocks
    /// will silently do nothing.
    pub fn disable_peripheral_clock(&mut self, pid: PeripheralID) {
        match pid {
            PeripheralID::PMC => (),  // Clock not under PMC control
            PeripheralID::EFC0 => (), // Clock not under PMC control
            PeripheralID::EFC1 => (), // Clock not under PMC control
            PeripheralID::UART => self.pmc_pcdr0.write_with_zero(|w| w.pid8().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::SDRAMC => self.pmc_pcdr0.write_with_zero(|w| w.pid10().set_bit()),
            PeripheralID::PIOA => self.pmc_pcdr0.write_with_zero(|w| w.pid11().set_bit()),
            PeripheralID::PIOB => self.pmc_pcdr0.write_with_zero(|w| w.pid12().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::PIOC => self.pmc_pcdr0.write_with_zero(|w| w.pid13().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::PIOD => self.pmc_pcdr0.write_with_zero(|w| w.pid14().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::PIOE => self.pmc_pcdr0.write_with_zero(|w| w.pid15().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::PIOF => self.pmc_pcdr0.write_with_zero(|w| w.pid16().set_bit()),
            PeripheralID::USART0 => self.pmc_pcdr0.write_with_zero(|w| w.pid17().set_bit()),
            PeripheralID::USART1 => self.pmc_pcdr0.write_with_zero(|w| w.pid18().set_bit()),
            PeripheralID::USART2 => self.pmc_pcdr0.write_with_zero(|w| w.pid19().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::USART3 => self.pmc_pcdr0.write_with_zero(|w| w.pid20().set_bit()),
            PeripheralID::HSMCI => self.pmc_pcdr0.write_with_zero(|w| w.pid21().set_bit()),
            PeripheralID::TWI0 => self.pmc_pcdr0.write_with_zero(|w| w.pid22().set_bit()),
            PeripheralID::TWI1 => self.pmc_pcdr0.write_with_zero(|w| w.pid23().set_bit()),
            PeripheralID::SPI0 => self.pmc_pcdr0.write_with_zero(|w| w.pid24().set_bit()),
            #[cfg(feature = "sam3x8h")]
            PeripheralID::SPI1 => self.pmc_pcdr0.write_with_zero(|w| w.pid25().set_bit()),
            PeripheralID::SSC => self.pmc_pcdr0.write_with_zero(|w| w.pid26().set_bit()),
            PeripheralID::TC0 => self.pmc_pcdr0.write_with_zero(|w| w.pid27().set_bit()),
            PeripheralID::TC1 => self.pmc_pcdr0.write_with_zero(|w| w.pid28().set_bit()),
            PeripheralID::TC2 => self.pmc_pcdr0.write_with_zero(|w| w.pid29().set_bit()),
            PeripheralID::TC3 => self.pmc_pcdr0.write_with_zero(|w| w.pid30().set_bit()),
            PeripheralID::TC4 => self.pmc_pcdr0.write_with_zero(|w| w.pid31().set_bit()),
            PeripheralID::TC5 => self.pmc_pcdr1.write_with_zero(|w| w.pid32().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC6 => self.pmc_pcdr1.write_with_zero(|w| w.pid33().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC7 => self.pmc_pcdr1.write_with_zero(|w| w.pid34().set_bit()),
            #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
            PeripheralID::TC8 => self.pmc_pcdr1.write_with_zero(|w| w.pid35().set_bit()),
            PeripheralID::PWM => self.pmc_pcdr1.write_with_zero(|w| w.pid36().set_bit()),
            PeripheralID::ADC => self.pmc_pcdr1.write_with_zero(|w| w.pid37().set_bit()),
            PeripheralID::DACC => self.pmc_pcdr1.write_with_zero(|w| w.pid38().set_bit()),
            PeripheralID::DMAC => self.pmc_pcdr1.write_with_zero(|w| w.pid39().set_bit()),
            PeripheralID::UOTGHS => self.pmc_pcdr1.write_with_zero(|w| w.pid40().set_bit()),
            PeripheralID::TRNG => self.pmc_pcdr1.write_with_zero(|w| w.pid41().set_bit()),
            #[cfg(feature = "sam3x")]
            PeripheralID::EMAC => self.pmc_pcdr1.write_with_zero(|w| w.pid42().set_bit()),
            PeripheralID::CAN0 => self.pmc_pcdr1.write_with_zero(|w| w.pid43().set_bit()),
            PeripheralID::CAN1 => self.pmc_pcdr1.write_with_zero(|w| w.pid44().set_bit()),
        }
    }
}
