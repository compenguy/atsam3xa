# ATSAM3X/A Hardware Abstraction Layer


This crate abstracts target initialization and configuration, implementing
traits from the [embedded-hal](https://crates.io/crates/embedded-hal) crate
for the [ATSAM3X/A family of Atmel MCUs](https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-11057-32-bit-Cortex-M3-Microcontroller-SAM3X-SAM3A_Datasheet.pdf#G19.1090731).

## Status

Target support:

| target | progress |
| ------ | -------- |
| atsam3a4c | untested |
| atsam3a4c | untested |
| atsam3a8c | untested |
| atsam3x4c | untested |
| atsam3x4e | untested |
| atsam3x8c | untested |
| atsam3x8e | simple applications running on Arduino Due |
| atsam3x8h | untested |

Peripheral support:

| peripheral | support level | notes |
| ---------- | ------------- | ----- |
| PMC/SUPC | clocking only | known issue affecting PLL configuration |
| MATRIX | SYSIO control only | allows switching ERASE pin to PC0 |
| SYST | Delay (sleep) support | |
| EFC0/1 | Configure op cycle time only | |
| PIOA-PIOF | switch between periph A/B/GPIO, pin config, driving/reading | |
| WDT | mostly complete | only disablement has been tested |

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

