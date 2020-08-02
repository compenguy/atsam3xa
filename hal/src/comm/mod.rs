//! Working with the U(S)ART peripherals.
//!
//! The atsam3x/a hardware has several U(S)ARTs that can
//! be configured to perform a variety of serial communication
//! modes (RS-232, RS-485, SPI, LIN, etc).  This configuration
//! is expressed through the use of type states to make it
//! difficult to misuse.
// SPI Flr|  MOSI  |  MISO  | Clock  |        |   NSS  |
// SPI Ldr|  MISO  |  MOSI  | Clock  |   NSS  |        |
// RS-232 |   RX   |   TX   |        |   RTS  |   CTS  | Periph ID |
// -------+--------+--------+--------+--------+--------+-----------|
// UART   | PA08/A | PA09/A |        |        |        |     8     |
// USART0 | PA10/A | PA11/A | PA17/B | PB25/A | PB26/A |    17     | LIN
// USART1 | PA12/A | PA13/A | PA16/A | PA14/A | PA15/A |    18     |
// USART2 | PB21/A | PB20/A | PB24/A | PB22/A | PB23/A |    19     |
// USART3 | PD05/B | PD04/B | PE16/B | PF05/A | PF04/A |    20     |

mod uart;
// mod rs485;
// mod lin;
// mod spi;
// mod irda;

pub use self::uart::*;
// pub use self::rs485::*;
// pub use self::lin::*;
// pub use self::spi::*;
// pub use self::irda::*;
