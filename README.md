# NES emulator

A NES emulator written in Rust.
The purpose of this project is to learn how the NES works and implement a playable emulator.

## Status of the project
The project is under development. 

- [ ] CPU.
  - [ ] nestest.
    Currently the nestest passes when CPU & PPU cycles are not taken into account i.e. instructions work almost as expected. WIP
- [ ] PPU. WIP
- [ ] APU


## Building the project

### Prerequisites

Latest Rust stable and SDL2

### Build instructions
```
cargo build
```

## Running tests

The emulator tests instructions of the CPU using the [nestest.rom](http://nickmass.com/images/nestest.nes). The nestest.rom needs to be inside folder 'tests' for the test to be able to work.

Tests can be run with 
```
cargo test
```

