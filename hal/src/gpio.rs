//! Configuring the GPIO pins
use crate::hal::digital::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};

// SAM3A4C, SAM3A8C, SAM3X4C, and SAM3X8C (100-pin) only have PIOA-PIOB
#[cfg(feature = "sam3_c")]
use crate::target_device::{PIOA, PIOB};

// SAM3X4E and SAM3X8E (144-pin) have PIOA-PIOD
#[cfg(feature = "sam3_e")]
use crate::target_device::{PIOA, PIOB, PIOC, PIOD};

// SAM3X8H has 217 pins and PIOA-PIOF
#[cfg(feature = "sam3x8h")]
use crate::target_device::{PIOA, PIOB, PIOC, PIOD, PIOE, PIOF};

use core::marker::PhantomData;

/// PIO controller configuration register block.
pub struct PioGroup<PIOn> {
    group: PIOn,
}

impl<PIOn> core::ops::Deref for PioGroup<PIOn> {
    type Target = PIOn;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl<PIOn> core::ops::DerefMut for PioGroup<PIOn> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group
    }
}

/// Represents a pin in an unconfigured state.
pub struct Unconfigured;

/// Represents a pin configured for input.
/// The MODE type is typically one of `Floating` or `PullUp`.
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Represents a pin configured for output.
/// The MODE type is typically one of `PushPull`, or
/// `OpenDrain`.
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

// The following collection of types is used to encode the
// state of the pin at compile time and helps to avoid misuse.

/// Floating Input
pub struct Floating;
/// Pulled up Input
pub struct PullUp;

/// Totem Pole aka Push-Pull
pub struct PushPull;
/// Open drain output
pub struct OpenDrain;

/// Peripheral Function A
pub struct PfA;
/// Peripheral Function B
pub struct PfB;

macro_rules! pin {
    (
        $group:ident,
        $PinType:ident,
        $pin_ident:ident,
        $pin_no:expr
    ) => {
        crate::paste! {
        /// Represents the IO pin with the matching name.
        pub struct $PinType<MODE> {
            _mode: PhantomData<MODE>,
        }
        } // end paste

        crate::paste! {
        /// Represents an unconfigured IO pin.
        pub type [<Unconfigured $PinType>] = $PinType<Unconfigured>;
        impl Default for [<Unconfigured $PinType>] {
            fn default() -> Self {
                Self::new()
            }
        }

        impl [<Unconfigured $PinType>] {
            /// Instantiate the pin in an unconfigured state
            pub fn new() -> Self {
                Self {
                    _mode: PhantomData,
                }
            }
        }
        } // end paste

        impl<MODE> $PinType<MODE> {
            /// Configures the pin to operate as a floating input
            pub fn into_floating_input(self) -> $PinType<Input<Floating>> {
                crate::paste! {
                // Enable PIO (not peripheral) mode
                unsafe {(*$group::ptr()).per.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Disable output mode (input)
                unsafe {(*$group::ptr()).odr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Disable pullup mode (floating)
                unsafe {(*$group::ptr()).pudr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as a pulled-up input
            pub fn into_pull_up_input(self) -> $PinType<Input<PullUp>> {
                crate::paste! {
                // Enable PIO (not peripheral) mode
                unsafe {(*$group::ptr()).per.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Disable output mode (input)
                unsafe {(*$group::ptr()).odr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Enable pullup mode
                unsafe {(*$group::ptr()).puer.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as an open-drain output
            pub fn into_open_drain_output(self) -> $PinType<Output<OpenDrain>> {
                crate::paste! {
                // Enable PIO (not peripheral) mode
                unsafe {(*$group::ptr()).per.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Enable output mode
                unsafe {(*$group::ptr()).oer.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Enable open-drain (multi-driver) mode
                unsafe {(*$group::ptr()).mder.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as an push-pull output
            pub fn into_push_pull_output(self) -> $PinType<Output<PushPull>> {
                crate::paste! {
                // Enable PIO (not peripheral) mode
                unsafe {(*$group::ptr()).per.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Enable output mode
                unsafe {(*$group::ptr()).oer.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Disable open-drain mode
                unsafe {(*$group::ptr()).mddr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Disable pull-up mode
                unsafe {(*$group::ptr()).pudr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to function as the primary (A) attached peripheral.
            pub fn into_peripheral_a(self) -> $PinType<PfA> {
                crate::paste! {
                // Disable PIO (enable peripheral) mode
                unsafe {(*$group::ptr()).pdr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Select Peripheral A (0 -> A, 1 -> B)
                unsafe {(*$group::ptr()).absr.modify(|_, w| w.[<p $pin_no>]().clear_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to function as the alternate (B) attached peripheral.
            pub fn into_peripheral_b(self) -> $PinType<PfB> {
                crate::paste! {
                // Disable PIO (enable peripheral) mode
                unsafe {(*$group::ptr()).pdr.write_with_zero(|w| w.[<p $pin_no>]().set_bit());}
                // Select Peripheral B (0 -> A, 1 -> B)
                unsafe {(*$group::ptr()).absr.modify(|_, w| w.[<p $pin_no>]().set_bit());}
                } // end paste
                $PinType { _mode: PhantomData }
            }
        }

        impl<MODE> $PinType<Output<MODE>> {
            /// Toggle the logic level of the pin; if it is currently high, set it low
            /// and vice-versa.
            pub fn toggle(&mut self) {
                self.toggle_impl();
            }

            fn toggle_impl(&mut self) {
                if self.is_high() {
                    self.set_low_impl();
                } else {
                    self.set_high_impl();
                }
            }

            /// Set the logic level of the pin high.
            pub fn set_high(&mut self) {
                self.set_high_impl();
            }

            fn set_high_impl(&mut self) {
                crate::paste! {
                unsafe {(*$group::ptr()).sodr.write_with_zero(|w| w.[<p $pin_no>]().bit(true));}
                } // end paste
            }

            /// Set the logic level of the pin low.
            pub fn set_low(&mut self) {
                self.set_low_impl();
            }

            fn set_low_impl(&mut self) {
                crate::paste! {
                unsafe {(*$group::ptr()).codr.write_with_zero(|w| w.[<p $pin_no>]().bit(true));}
                } // end paste
            }
        }

        impl<MODE> ToggleableOutputPin for $PinType<Output<MODE>> {
            type Error = core::convert::Infallible;

            fn try_toggle(&mut self) -> Result<(), Self::Error> {
                self.toggle_impl();
                Ok(())
            }
        }

        impl<MODE> OutputPin for $PinType<Output<MODE>> {
            type Error = core::convert::Infallible;

            fn try_set_high(&mut self) -> Result<(), Self::Error> {
                self.set_high_impl();
                Ok(())
            }

            fn try_set_low(&mut self) -> Result<(), Self::Error> {
                self.set_low_impl();
                Ok(())
            }
        }

        impl<MODE> StatefulOutputPin for $PinType<Output<MODE>> {
            fn try_is_set_high(&self) -> Result<bool, Self::Error> {
                Ok(self.is_high())
            }

            fn try_is_set_low(&self) -> Result<bool, Self::Error> {
                Ok(self.is_low())
            }
        }

        impl<MODE> $PinType<MODE> {
            /// Test the logic level of the pin; if it is currently high, return true.
            /// If $group is not currently clocked, this will return the value from
            /// when it was last clocked.
            pub fn is_high(&self) -> bool {
                crate::paste! {
                unsafe {(*$group::ptr()).pdsr.read().[<p $pin_no>]().bits()}
                } // end paste
            }

            /// Test the logic level of the pin; if it is currently low, return true.
            /// If $group is not currently clocked, this will return the value from
            /// when it was last clocked.
            pub fn is_low(&self) -> bool {
                !self.is_high()
            }
        }

        impl<MODE> InputPin for $PinType<MODE> {
            type Error = core::convert::Infallible;

            fn try_is_high(&self) -> Result<bool, Self::Error> {
                Ok(self.is_high())
            }

            fn try_is_low(&self) -> Result<bool, Self::Error> {
                Ok(self.is_low())
            }
        }
    };
} // End `pin` macro definition

macro_rules! pio_group {
    (
        $group_id:ident,
        $peripheral_id:expr,
        [
          $($pin_no:expr,)+
        ]
    ) => {

crate::paste! {
impl PioGroup<[<PIO $group_id:upper>]> {
    /// Return the Peripheral ID for the PIOF Controller, used for controlling
    /// clocks and interrupts.
    pub fn peripheral_id() -> $crate::clock::PeripheralID {
        $crate::clock::PeripheralID::[<Id $peripheral_id Pio $group_id:upper>]
    }

    /// Instantiate a representation of a PIO group, providing an interface
    /// to all the pins it controls.
    pub fn new(group: [<PIO $group_id:upper>]) -> Self {
        Self {
            group,
        }
    }

    $(
    /// Access the configuration and state of the pin of this name on this PIO
    /// group.
    pub fn [<p $pin_no>](&self) -> [<Unconfigured P $group_id:lower $pin_no>] {
        [<Unconfigured P $group_id:lower $pin_no>]::new()
    }
    )+

}

impl From<[<PIO $group_id:upper>]> for PioGroup<[<PIO $group_id:upper>]> {
    fn from(group: [<PIO $group_id:upper>]) -> Self {
        Self::new(group)
    }
}

$(
    pin!([<PIO $group_id:upper>], [<P $group_id:lower $pin_no>], [<p $group_id:lower $pin_no>], $pin_no);
)+
} // end paste
    };
} // End `group` macro definition

// PIOA has the same pin set among all targets
pio_group!(
    a,
    11,
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ]
);

// PIOB has the same pin set among all targets
pio_group!(
    b,
    12,
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ]
);

// PIOC pins 0-30 on the atsam3x4e, atsam3x8e, and atsam3x8h targets
#[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
pio_group!(
    c,
    13,
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30,
    ]
);

// PIOD is not supported by the atsam3x?c targets, and only has pins 0-10 on
// the atsam3x?e targets
#[cfg(feature = "sam3_e")]
pio_group!(d, 14, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,]);

// PIOD has pins 0-30 on the atsam3x8h target
#[cfg(feature = "sam3x8h")]
pio_group!(
    d,
    14,
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30,
    ]
);

// PIOE has pins 0-31 on the atsam3x8h target
#[cfg(feature = "sam3x8h")]
pio_group!(
    e,
    15,
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ]
);

// PIOF has pins 0-6 on the atsam3x8h target
#[cfg(feature = "sam3x8h")]
pio_group!(f, 16, [0, 1, 2, 3, 4, 5, 6,]);

/// This macro is a helper for defining a `Pins` type in a board support
/// crate.  This type is used to provide more meaningful aliases for the
/// various GPIO pins for a given board.
#[macro_export]
macro_rules! define_pins {
    (
        $(#[$topattr:meta])* struct $Type:ident,
        $( $(#[$attr:meta])* pin $name:ident = $pio_group:ident : $pin_ident:expr),+ ,
    ) => {

$crate::paste! {
$(#[$topattr])*
pub struct $Type {
    /// Opaque reference to PIO controller A
    pub pa: $crate::gpio::PioGroup<$crate::target_device::PIOA>,
    /// Opaque reference to PIO controller B
    pub pb: $crate::gpio::PioGroup<$crate::target_device::PIOB>,
    /// Opaque reference to PIO controller C
    #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
    pub pc: $crate::gpio::PioGroup<$crate::target_device::PIOC>,
    /// Opaque reference to PIO controller D
    #[cfg(any(feature = "sam3_e", feature = "sam3x8h"))]
    pub pd: $crate::gpio::PioGroup<$crate::target_device::PIOD>,
    /// Opaque reference to PIO controller E
    #[cfg(feature = "sam3x8h")]
    pub pe: $crate::gpio::PioGroup<$crate::target_device::PIOE>,
    /// Opaque reference to PIO controller F
    #[cfg(feature = "sam3x8h")]
    pub pf: $crate::gpio::PioGroup<$crate::target_device::PIOF>,

    $(
        $(#[$attr])*
        pub $name: $crate::gpio::[<P $pio_group $pin_ident>]<$crate::gpio::Unconfigured>
    ),+
}
} // end paste

impl $Type {
    $crate::paste! {
    #[cfg(feature = "sam3_c")]
    /// Returns the pins for the device
    pub fn new(pioa: $crate::target_device::PIOA, piob: $crate::target_device::PIOB) -> Self {
        let pa = $crate::gpio::PioGroup::from(pioa);
        let pb = $crate::gpio::PioGroup::from(piob);
        $(
        let $name = [<p $pio_group>].[<p $pin_ident>]();
        )+
        $Type {
            pa,
            pb,
            $($name),+,
        }
    }
    } // end paste

    $crate::paste! {
    #[cfg(feature = "sam3_e")]
    /// Returns the pins for the device
    pub fn new(pioa: $crate::target_device::PIOA, piob: $crate::target_device::PIOB, pioc: $crate::target_device::PIOC, piod: $crate::target_device::PIOD) -> Self {
        let pa = $crate::gpio::PioGroup::from(pioa);
        let pb = $crate::gpio::PioGroup::from(piob);
        let pc = $crate::gpio::PioGroup::from(pioc);
        let pd = $crate::gpio::PioGroup::from(piod);
        $(
        let $name = [<p $pio_group>].[<p $pin_ident>]();
        )+
        $Type {
            pa,
            pb,
            pc,
            pd,
            $($name),+,
        }
    }
    } // end paste

    $crate::paste! {
    #[cfg(feature = "sam3x8h")]
    /// Returns the pins for the device
    pub fn new(pioa: $crate::target_device::PIOA, piob: $crate::target_device::PIOB, pioc: $crate::target_device::PIOC, piod: $crate::target_device::PIOD,  pioe: $crate::target_device::PIOE, piof: $crate::target_device::PIOF) -> Self {
        let pa = $crate::gpio::PioGroup::from(pioa);
        let pb = $crate::gpio::PioGroup::from(piob);
        let pc = $crate::gpio::PioGroup::from(pioc);
        let pd = $crate::gpio::PioGroup::from(piod);
        let pe = $crate::gpio::PioGroup::from(pioe);
        let pf = $crate::gpio::PioGroup::from(piof);
        $(
        let $name = [<p $pio_group>].[<p $pin_ident>]();
        )+
        $Type {
            pa,
            pb,
            pc,
            pd,
            pe,
            pf,
            $($name),+,
        }
    }
    } // end paste
}

}}
