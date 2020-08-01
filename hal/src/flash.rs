//! Configuring the embedded flash controllers.
use crate::target_device;
use target_device::{EFC0, EFC1};

/// Flash controller
pub struct FlashController<EFCn> {
    efc: EFCn,
}

impl<E> core::ops::Deref for FlashController<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.efc
    }
}

impl<E> core::ops::DerefMut for FlashController<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.efc
    }
}

/// Embedded flash controller 0 (EFC0)
pub type FlashController0 = FlashController<EFC0>;
impl FlashController<EFC0> {
    /// Instantiate a flash controller object, and initialize it to a safe value
    pub fn new(efc: EFC0) -> Self {
        let mut tmp = Self { efc };
        // Init flash wait state to a safe value
        tmp.set_op_cycle_count(4);
        tmp
    }

    /// Set the number of additional cycles to wait for read/write operations
    /// to complete.
    pub fn set_op_cycle_count(&mut self, count: u8) {
        self.efc.fmr.modify(|_, w| unsafe { w.fws().bits(count) });
    }

    /// Return the number of additional cycles to wait for read/write
    /// operations to complete.
    pub fn get_op_cycle_count(&self) -> u8 {
        self.efc.fmr.read().fws().bits()
    }
}

impl From<EFC0> for FlashController0 {
    fn from(efc0: EFC0) -> Self {
        Self::new(efc0)
    }
}

/// Embedded flash controller 1 (EFC1)
pub type FlashController1 = FlashController<EFC1>;
impl FlashController<EFC1> {
    /// Instantiate a flash controller object, and initialize it to a safe value
    pub fn new(efc: EFC1) -> Self {
        let mut tmp = Self { efc };
        // Init flash wait state to a safe value
        tmp.set_op_cycle_count(4);
        tmp
    }

    /// Set the number of additional cycles to wait for read/write operations
    /// to complete.
    pub fn set_op_cycle_count(&mut self, count: u8) {
        self.efc.fmr.modify(|_, w| unsafe { w.fws().bits(count) });
    }

    /// Return the number of additional cycles to wait for read/write
    /// operations to complete.
    pub fn get_op_cycle_count(&self) -> u8 {
        self.efc.fmr.read().fws().bits()
    }
}

impl From<EFC1> for FlashController1 {
    fn from(efc1: EFC1) -> Self {
        Self::new(efc1)
    }
}
