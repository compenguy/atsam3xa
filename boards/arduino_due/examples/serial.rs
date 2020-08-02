#![no_std]
#![no_main]

use arduino_due as board;

use board::entry;
use board::prelude::*;

use board::hal::clock::SystemClocks;
use board::hal::comm;
use board::hal::delay::Delay;
use board::hal::watchdog::WdtBuilder;
use board::pac::{CorePeripherals, Peripherals};
use board::hal::time::Hertz;

use core::fmt::Write;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = SystemClocks::new(peripherals.PMC, peripherals.SUPC);
    let _ = WdtBuilder::from(peripherals.WDT).disable();
    let pins = board::Pins::new(
        peripherals.PIOA,
        peripherals.PIOB,
        peripherals.PIOC,
        peripherals.PIOD,
    );
    let mut delay = Delay::new(core.SYST, clocks.get_syscore());

    let mut led_l = pins.led_l.into_push_pull_output();

    let mut uart = comm::Uart0::new(
        peripherals.UART,
        pins.d0_rx0.into_peripheral_a(),
        pins.d1_tx0.into_peripheral_a(),
    );
    uart.set_baudrate(Hertz(57600), clocks.get_syscore());

    loop {
        led_l.set_high();
        delay.try_delay_ms(200u8).unwrap();
        uart.write_str("A").unwrap();

        led_l.set_low();
        delay.try_delay_ms(200u8).unwrap();
        uart.write_str("B").unwrap();
    }
}
