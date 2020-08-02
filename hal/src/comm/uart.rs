use crate::hal::blocking;
use crate::hal::serial;
use crate::target_device::UART;
use embedded_hal::prelude::*;
//use crate::target_device::{UART, USART0, USART1, USART2, USART3};
use crate::gpio::{Pa8, Pa9, PfA};
use crate::target_device::uart::mr::{CHMODE_A, PAR_A};
use crate::time::Hertz;
use core::fmt;

/// UART controller configuration
pub struct Uart<UartP, RX, TX, RTS, CTS> {
    /// U(S)ART peripheral from the PAC
    uart_p: UartP,
    /// Serial RX pin
    _rx: RX,
    /// Serial TX pin
    _tx: TX,
    /// Serial Request-to-send pin, if any
    _rts: RTS,
    /// Serial Clear-to-send pin, if any
    _cts: CTS,
}

impl<UartP, RX, TX, RTS, CTS> core::ops::Deref for Uart<UartP, RX, TX, RTS, CTS> {
    type Target = UartP;

    fn deref(&self) -> &Self::Target {
        &self.uart_p
    }
}

impl<UartP, RX, TX, RTS, CTS> core::ops::DerefMut for Uart<UartP, RX, TX, RTS, CTS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uart_p
    }
}

//        |   RX   |   TX   |   RTS  |   CTS  | Periph ID |
// -------+--------+--------+--------+--------+-----------|
// UART   | PA08/A | PA09/A |        |        |     8     |
// USART0 | PA10/A | PA11/A | PB25/A | PB26/A |    17     |
// USART1 | PA12/A | PA13/A | PA14/A | PA15/A |    18     |
// USART2 | PB21/A | PB20/A | PB22/A | PB23/A |    19     |
//
// atsam3_e:
// USART3 | PD05/B | PD04/B |        |        |    20     |
// atsam3x8h:
// USART3 | PD05/B | PD04/B | PF05/A | PF04/A |    20     |

/// The Uart derived from the UART peripheral, using pins Pa8 and Pa9 in
/// peripheral A mode.  No hardware flow control.
pub type Uart0 = Uart<UART, Pa8<PfA>, Pa9<PfA>, (), ()>;

impl Uart0 {
    /// Instantiate a representation of a UART, providing an interface
    /// configure, send, and receive on it.
    pub fn new(uart_p: UART, rx: Pa8<PfA>, tx: Pa9<PfA>) -> Self {
        let uart0 = Self {
            uart_p,
            _rx: rx,
            _tx: tx,
            _rts: (),
            _cts: (),
        };
        uart0
            .cr
            .write_with_zero(|w| w.rxen().set_bit().txen().set_bit().rststa().set_bit());
        uart0
    }

    /// Set the serial line parity error correcting strategy.
    pub fn set_parity(&mut self, parity: PAR_A) {
        self.mr.write(|w| w.par().variant(parity));
    }

    /// Get the serial line parity error correcting strategy.
    pub fn get_parity(&self) -> PAR_A {
        match self.mr.read().par().variant() {
            crate::target_device::generic::Variant::Val(v) => v,
            crate::target_device::generic::Variant::Res(_) => unreachable!(),
        }
    }

    /// Set the serial channel echo/loopback mode.
    pub fn set_channel_mode(&mut self, ch_mode: CHMODE_A) {
        self.mr.write(|w| w.chmode().variant(ch_mode));
    }

    /// Get the serial channel echo/loopback mode.
    pub fn get_channel_mode(&self) -> CHMODE_A {
        self.mr.read().chmode().variant()
    }

    /// Set the serial line baud rate, which is configured to be a fraction of
    /// the master clock speed.
    pub fn set_baudrate<I: Into<Hertz>>(&mut self, baud_rate: I, mck: I) {
        // cd = mck/(16*baudrate)
        let cd = mck.into().0 / (baud_rate.into().0 << 4);
        self.brgr.write(|w| unsafe { w.cd().bits(cd as u16) });
    }

    /// Return the serial line baud rate, calculated to be a fraction of the
    /// master clock speed.
    pub fn get_baudrate<I: Into<Hertz>>(&self, mck: I) -> Hertz {
        // baudrate = mck/(16*cd)
        Hertz(mck.into().0 / ((self.brgr.read().cd().bits() as u32) << 4))
    }
}

impl From<(UART, Pa8<PfA>, Pa9<PfA>)> for Uart0 {
    fn from(parts: (UART, Pa8<PfA>, Pa9<PfA>)) -> Self {
        Self::new(parts.0, parts.1, parts.2)
    }
}

impl serial::Write<u8> for Uart0 {
    type Error = core::convert::Infallible;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        unsafe {
            if !self.sr.read().txrdy().bits() {
                return Err(nb::Error::WouldBlock);
            }

            self.thr.write_with_zero(|w| w.txchr().bits(word));
        }

        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        if !self.sr.read().txrdy().bits() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }
}

impl serial::Read<u8> for Uart0 {
    type Error = core::convert::Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        if !self.sr.read().rxrdy().bits() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(self.rhr.read().rxchr().bits())
    }
}

impl blocking::serial::write::Default<u8> for Uart0 {}

impl fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_bwrite_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}
