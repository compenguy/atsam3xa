//! HAL for the ATSAM3X family of microcontrollers
//!
//! This is an implementation of the [`embedded-hal`] traits for the ATSAM3X family of
//! microcontrollers.
//!
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! # Requirements
//!
//! This crate requires `arm-none-eabi-gcc` to be installed and available in `$PATH` to build.
//!
//! # Usage
//!
//! To build applications (binary crates) using this crate follow the [cortex-m-quickstart]
//! instructions and add this crate as a dependency in step number 5 and make sure you enable the
//! "rt" Cargo feature of this crate.
//!
//! [cortex-m-quickstart]: https://docs.rs/cortex-m-quickstart/~0.3
//!
//! # Examples
//!
//! Examples of *using* these abstractions can be found in the documentation of the [`f3`] crate.
//!
//! [`f3`]: https://docs.rs/f3/~0.6

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

pub use paste::paste;

#[cfg(feature = "sam3a4c")]
pub use atsam3a4c as target_device;
#[cfg(feature = "sam3a8c")]
pub use atsam3a8c as target_device;
#[cfg(feature = "sam3x4c")]
pub use atsam3x4c as target_device;
#[cfg(feature = "sam3x4e")]
pub use atsam3x4e as target_device;
#[cfg(feature = "sam3x8c")]
pub use atsam3x8c as target_device;
#[cfg(feature = "sam3x8e")]
pub use atsam3x8e as target_device;
#[cfg(feature = "sam3x8h")]
pub use atsam3x8h as target_device;

pub use embedded_hal as hal;

pub mod bus;
pub mod clock;
pub mod comm;
pub mod delay;
pub mod flash;
pub mod gpio;
pub mod prelude;
pub mod time;
pub mod watchdog;

/// Identifier used for enabling/disabling the clock to that peripheral, as
/// well as for controlling the peripher interrupt in the NVIC. Peripherals
/// 0-8, and 10 are always clocked.
pub type PeripheralID = target_device::Interrupt;
