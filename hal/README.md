# ATSAM3X/A Hardware Abstraction Layer
===
This crate abstracts target initialization and configuration, implementing
traits from the `[embedded-hal](https://crates.io/crates/embedded-hal)` crate
for the [ATSAM3X/A family of Atmel MCUs](https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-11057-32-bit-Cortex-M3-Microcontroller-SAM3X-SAM3A_Datasheet.pdf#G19.1090731).

## Status

Target support:

* atsam3a4c - *in work* *untested*
* atsam3a8c - *in work* *untested*
* atsam3x4c - *in work* *untested*
* atsam3x4e - *in work* *untested*
* atsam3x8c - *in work* *untested*
* atsam3x8e - *in work*
* atsam3x8h - *in work* *untested*

Peripheral support:

* PMC/SUPC - clocking only (known issue affecting PLL configuration)
* MATRIX - SYSIO control only (switch ERASE pin to PC0)
* SYST - Delay (sleep) support
* EFC0/1 - Setting operation cycle time only
* PIOA-PIOF - Peripheral/GPIO toggling, pin configuration, driving/reading
* WDT - support pretty much complete, but only disablement has been tested

# Credits and Licensing

Many thanks to Michal Fita of the
[atsams70-rust](https://github.com/michalfita/atsams70-rust) project, and 
Wez Furlong, Paul Sajna, Michael van Niekerk, and Jesse Braham of the
[atsamd](https://github.com/atsamd-rs/atsamd) project.  I have learned so much
by studying how these projects support other Atmel SAM target families, and
I wouldn't have been able to do it without their work.

The licensing of this crate reflects the impact their work has had on this
project, being the sum of BSD0 from the atsamx7x-hal crate, and either the
MIT or Apache-2.0 license from the atsamd-hal crate, represented by the
SPDX license expression:

`BSD0 AND (MIT OR Apache-2.0)`

