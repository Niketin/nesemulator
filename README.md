# NES emulator

A NES emulator written in Rust.

The purpose of this project is to learn how the NES works and implement a playable emulator.

## Features

- Supports 1 player
- Tested playable roms
  - Donkey Kong
  - Donkey Kong Jr.
  - Balloon Fight
- No sound
- Passes nestest
- Mappers
  - NROM


## Controls

| NES controller button | Keyboard key |
| --- | --- |
| A | Z |
| B | X |
| Start | N |
| Select | M |
| Up | Up |
| Down | Down |
| Left | Left |
| Right | Right |

## Building the project

### Prerequisites

Latest Rust stable and SDL2

### Build instructions
```
cargo build --release
```

## Running rom

```
cargo run --release -- <path_to_rom>
```

## Running tests

The emulator tests instructions of the CPU using the [nestest.rom](http://nickmass.com/images/nestest.nes). The nestest.rom needs to be inside folder 'tests' for the test to be able to work.

Tests can be run with 
```
cargo test
```
