# ATSAM3X/A Support

This repository contains low-level Peripheral Access Crates (./pac), a
Hardware Abstraction Layer (./hal), and Board support definitions (./boards).

## PACs

### Status

* atsam3a4c - *complete*
* atsam3a8c - *complete*
* atsam3x4c - *complete*
* atsam3x4e - *complete*
* atsam3x8c - *complete*
* atsam3x8e - *complete*
* atsam3x8h - *complete*

### Updates

The PAC support is generated from the Atmel Microchip Packs SVDs by:

* downloading the sam3x atpack file (`Atmel SAM3X Series Device
  Support (1.0.50)`)
* downloading the sam3a atpack file (`Atmel SAM3A Series Device
  Support (1.0.50)`)
* unzipping them
* copying the entire contents of the svd directory from the zip output into
  the svd subdirectory of this project
* running the `generate.sh` script located in the root of this project

This should regenerate all of the `pac/atsam*` folders with code supporting
the provided register definitions.

For usage, license information, etc see the README.md for the PAC for your part.

## HAL

For project status, usage, license information, etc see the README.md for the
atsam3xa-hal crate.

## Boards

Support for the Arduino Due board is forthcoming.

