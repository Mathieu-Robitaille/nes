## NES

Here's another NES emulator, this project is an experiment in handling a sprawling codebase and organization. Some things may not be optimal, some things may be gross.

Currently the project is in the `Just get it working phase` so it's a bit all over the place.

### PPU
---
I will never forget writing this ppu.
Nothing has broken me more.

### Status
---

- This needs to get filled out more
- [] Logging
    - [] To file
    - [] Log levels
- [-] Cpu
    - [x] Instruction table
    - [x] Instructions
    - [?] Correct cpu clock timing
- [-] PPU
    - [x] OAM DMA
    - [x] Sprite tables
    - [x] Background
        - [x] Name Table
        - [x] Palettes
        - [x] Registers
    - [] Sprites
        - [] 8 x 8
        - [] 8 x 16
- [] Game pad
- [] APU
    - [] Square Wave 1
    - [] Square Wave 2
    - [] Triangle Wave
    - [] Noise
    - [] DMC
- [x] Cartridge
    - [x] Read from rom file
- [-] Mappers
    - [x] 001
- [x] Debug
- [] Emulation features
    - [] Snapshots
        - [] Read/Write state from/to file
    - [] ???
- [] Tests
    - [?] PPU
    - [] CPU
    - [] ???

----
### Resources:
 - https://www.nesdev.org/wiki/Nesdev_Wiki
 - [The man himself](https://github.com/OneLoneCoder/olcNES)