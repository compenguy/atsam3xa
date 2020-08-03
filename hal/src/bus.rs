//! Configuring the bus matrix interconnects.
use crate::target_device;
use target_device::MATRIX;

/// Bus interconnect configuration register block.
pub struct BusInterconnect {
    matrix: MATRIX,
}

impl core::ops::Deref for BusInterconnect {
    type Target = MATRIX;

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl core::ops::DerefMut for BusInterconnect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matrix
    }
}

impl BusInterconnect {
    /// Instantiate a bus interconnect controller object, and initialize it to
    /// a safe value
    pub fn new(matrix: MATRIX) -> Self {
        let mut tmp = Self { matrix };
        // Disable the System I/O configuration for the ERASE pin, exposing it
        // for use by PC0
        tmp.disable_sysio();
        tmp
    }

    /// Toggle off the sysio configurations for pins, enabling their peripheral
    /// configurations instead.
    pub fn disable_sysio(&mut self) {
        self.ccfg_sysio.modify(|_, w| w.sysio12().clear_bit());
    }

    /// Toggle on the sysio configurations for pins, disabling their peripheral
    /// configurations.
    pub fn enable_sysio(&self) {
        self.ccfg_sysio.modify(|_, w| w.sysio12().set_bit());
    }
}

impl From<MATRIX> for BusInterconnect {
    fn from(matrix: MATRIX) -> Self {
        Self::new(matrix)
    }
}
