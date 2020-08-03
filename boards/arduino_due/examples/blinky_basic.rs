#![no_std]
#![no_main]

use arduino_due as board;

use board::entry;
use board::prelude::*;

use board::pac::{CorePeripherals, Peripherals};
use board::hal::clock::SystemClocks;
use board::hal::delay::Delay;
use board::hal::watchdog::WdtBuilder;

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
    let mut led = pins.led_l.into_push_pull_output();
    let mut delay = Delay::new(core.SYST, clocks.get_syscore());

    loop {
        delay.try_delay_ms(200u8).unwrap();
        led.set_high();
        delay.try_delay_ms(200u8).unwrap();
        led.set_low();
    }
}
