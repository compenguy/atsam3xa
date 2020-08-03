#![no_std]

pub use atsam3xa_hal as hal;
pub use hal::target_device as pac;

pub use hal::prelude;

#[cfg(feature = "rt")]
extern crate cortex_m_rt;

#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

#[cfg(feature = "panic_halt")]
pub extern crate panic_halt;

use hal::define_pins;

// The docs could be further improved with details of the specific channels etc
define_pins!(
    /// Maps the pins to their arduino names and the numbers printed on the board.
    /// Information from: <https://github.com/arduino/ArduinoCore-sam/blob/master/variants/arduino_due_x/variant.cpp>
    struct Pins,

    /// Digital 0, UART RX0 (UART/Serial0)
    pin d0_rx0 = a : 8,

    /// Digital 1, UART TX0 (UART/Serial0)
    pin d1_tx0 = a : 9,

    /// Digital 2, TIOA0
    pin d2_tioa0 = b : 25,

    /// Digital 3, TIOA7
    pin d3_tioa7 = c : 28,

    /// Digital 4, TIOB6
    pin d4_tiob6 = c : 26,

    /// Digital 5, TIOA6
    pin d5_tioa6 = c : 25,

    /// Digital 6, PWML7
    pin d6_pwml7 = c : 24,

    /// Digital 7, PWML6
    pin d7_pwml6 = c : 23,

    /// Digital 8, PWML5
    pin d8_pwml5 = c : 22,

    /// Digital 9, PWML4
    pin d9_pwml4 = c : 21,

    /// Digital 10, NPCS0
    pin d10_npcs0 = a : 28,

    /// Digital 11, TIOA8
    pin d11_tioa8 = d : 7,

    /// Digital 12, TIOB8
    pin d12_tiob8 = d : 8,

    /// Digital 13 (shared with `led_l`), TIOB0
    pin d13_tiob0 = b : 27,

    /// L (AMBER LED) (shared with `d13_tiob0`)
    pin led_l = b : 27,

    /// Digital 14, TX3 (USART3/Serial3)
    pin d14_tx3 = d : 4,

    /// Digital 15, RX3 (USART3/Serial3)
    pin d15_rx3 = d : 5,

    /// Digital 16, TX2 (USART1/Serial2)
    pin d16_tx2 = a : 13,

    /// Digital 17, RX2 (USART1/Serial2)
    pin d17_rx2 = a : 12,

    /// Digital 18, TX1 (USART0/Serial1)
    pin d18_tx1 = a : 11,

    /// Digital 19, RX1 (USART0/Serial1)
    pin d19_rx1 = a : 10,

    /// Digital 20, SDA0, TWD1
    pin d20_sda0_twd1 = b : 12,

    /// Digital 21, SCL0, TWCK1
    pin d21_scl0_twck1 = b : 13,

    /// Digital 22
    pin d22 = b : 26,

    /// Digital 23
    pin d23 = a : 14,

    /// Digital 24
    pin d24 = a : 15,

    /// Digital 25
    pin d25 = d : 0,

    /// Digital 26
    pin d26 = d : 1,

    /// Digital 27
    pin d27 = d : 2,

    /// Digital 28
    pin d28 = d : 3,

    /// Digital 29
    pin d29 = d : 6,

    /// Digital 30
    pin d30 = d : 9,

    /// Digital 31
    pin d31 = a : 7,

    /// Digital 32
    pin d32 = d : 10,

    /// Digital 33
    pin d33 = c : 1,

    /// Digital 34
    pin d34 = c : 2,

    /// Digital 35
    pin d35 = c : 3,

    /// Digital 36
    pin d36 = c : 4,

    /// Digital 37
    pin d37 = c : 5,

    /// Digital 38
    pin d38 = c : 6,

    /// Digital 39
    pin d39 = c : 7,

    /// Digital 40
    pin d40 = c : 8,

    /// Digital 41
    pin d41 = c : 9,

    /// Digital 42
    pin d42 = a : 19,

    /// Digital 43
    pin d43 = a : 20,

    /// Digital 44
    pin d44 = c : 19,

    /// Digital 45
    pin d45 = c : 18,

    /// Digital 46
    pin d46 = c : 17,

    /// Digital 47
    pin d47 = c : 16,

    /// Digital 48
    pin d48 = c : 15,

    /// Digital 49
    pin d49 = c : 14,

    /// Digital 50
    pin d50 = c : 13,

    /// Digital 51
    pin d51 = c : 12,

    /// Digital 52
    pin d52 = b : 21,

    /// Digital 53
    pin d53 = b : 14,

    /// Analog 0
    pin a0 = a : 16,

    /// Analog 1
    pin a1 = a : 24,

    /// Analog 2
    pin a2 = a : 23,

    /// Analog 3
    pin a3 = a : 22,

    /// Analog 4, TIOB2
    pin a4_tiob2 = a : 6,

    /// Analog 5
    pin a5 = a : 4,

    /// Analog 6, TIOB1
    pin a6_tiob1 = a : 3,

    /// Analog 7, TIOA1
    pin a7_tioa1 = a : 2,

    /// Analog 8
    pin a8 = b : 17,

    /// Analog 9
    pin a9 = b : 18,

    /// Analog 10
    pin a10 = b : 19,

    /// Analog 11
    pin a11 = b : 20,

    /// Analog 12, DAC0
    pin a12_dac0 = b : 15,

    /// Analog 13, DAC1
    pin a13_dac1 = b : 16,

    /// Analog 14, CANRX
    pin a14_canrx = a : 1,

    /// Analog 15, CANTX
    pin a15_cantx = a : 0,

    /// SDA1, TWD0
    pin sda1_twd0 = a : 17,

    /// SCL1, TWCK0
    pin scl1_twck0 = a : 18,

    /// RX (AMBER LED)
    pin led_rx = c : 30,

    /// TX (AMBER LED)
    pin led_tx = a : 21,

    /// MISO
    pin miso = a : 25,

    /// MOSI
    pin mosi = a : 26,

    /// SCLK
    pin sclk = a : 27,

    /// NPCS0
    pin npcs0 = a : 28,

    /// NPCS3 (unconnected)
    pin npcs3 = b : 23,

    /// USB ID
    pin usb_id = b : 11,

    /// USB VBOF
    pin usb_vbof = b : 10,
);
