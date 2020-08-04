# Arduino Due Board Support Crate

This crate provides a type-safe API for working with the [Arduino Due board](https://store.arduino.cc/arduino-due).

## Examples
### Blinky Basic
#### Requirements
 - Use `arduino-cli` to fetch `bossac`
    - Download the latest release of [arduino-cli](https://github.com/arduino/arduino-cli/releases)
    - Run `arduino-cli config init`
    - Run `arduino-cli core install arduino:sam`
    - Tools will be placed in `~/.arduino15/packages/arduino/...`
 - Separately:
    - Linux - typically available in distribution package repositories (Debian/Ubuntu: `bossa-cli`)
    - Windows:
       - Arduino IDE installed
          - samd package installed
          - Now the arduino distribution contains bossac.exe in `ArduinoData/packages/arduino/tools/bossac/1.7.0/` add it to your path
          - Probably best to install an example sketch via the IDE just to make sure everything is working
 - thumbv7m-none-eabi rust target installed via `rustup target add thumbv7m-none-eabi`

#### Steps
```bash
$ cargo install make
$ cargo make flash
```

The default example to build and flash is `blinky_basic`.  To select a
different one, instead run:

```bash
$ cargo make --env TARGET_NAME=<example_name> flash
```

If the serial port name is wrong, the default can be overridden by setting
the `SERDEV_LINUX` or `SERDEV_WINDOWS` environment variable.
