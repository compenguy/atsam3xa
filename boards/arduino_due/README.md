# Arduino Due Board Support Crate

This crate provides a type-safe API for working with the [Arduino Due board](https://store.arduino.cc/arduino-due).

## Examples
### Blinky Basic
#### Requirements
 - Use `arduino-cli` to fetch `bossac` and the `arm-none-eabi` tools
    - Download the latest release of [arduino-cli](https://github.com/arduino/arduino-cli/releases)
    - Run `arduino-cli config init`
    - Run `arduino-cli core install arduino:sam`
    - Tools will be placed in `~/.arduino15/packages/arduino/...`
 - Separately:
    - Linux - typically available in distribution package repositories (Debian/Ubuntu: `bossa-cli`, `binutils-arm-none-eabi`)
    - Windows:
       - Arduino IDE installed
          - samd package installed
          - Now the arduino distribution contains bossac.exe in `ArduinoData/packages/arduino/tools/bossac/1.7.0/` add it to your path
          - Probably best to install an example sketch via the IDE just to make sure everything is working
       - arm-none-eabi tools installed, you need gcc and objcopy.
 - thumbv7m-none-eabi rust target installed via `rustup target add thumbv7m-none-eabi`

#### Steps
```bash
cargo build --release --example blinky_basic
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/examples/blinky_basic target/blinky_basic.bin
stty -F /dev/ttyACM0 speed 1200 cs8 -cstopb -parenb
bossac -i -d -U true -i -e -w -v target/blinky_basic.bin -R
```

