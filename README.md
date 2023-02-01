** NES **

Note that i'm still working on the arch of this, None of this is supposed to look clean right now
If you find yourself asking "why is this like this?", everythign is subject to change so there's no reason to invest in making something perfect yet.

Resources:
 - https://www.nesdev.org/wiki/Nesdev_Wiki
 - [The man himself](https://github.com/OneLoneCoder/olcNES)


Originally this was as close to a 1:1 port of javids c++ nes emulator as possible, using the rust bindings for olcgameengine. It's hit the point where it's a bit too messy to continue down that path.
As such I'm re-writing this to make more sense in rust, but to really get started on that I need to ensure certain things are working beforehand.


 ### Optimizing this project.
 ---
 Once we can load roms and play a game on this emulator I will start looking at optimizing it.
 I'm not starting with this because the architecture most likely will change as I add more "chips" to this projecct.


### Current issues
 - PPU outputs garbage data
    - This was roughly drawing correctly at one point, but I ripped out the union struct annoyance due to another issue. It's been broken since.
    - It seems around scanline 120 it starts to draw the screen, im not sure why yet
        - maybe the ppu read is at fault?
    - at scan line 120- 240 it draws garbage data
    - there seems to be a slight X offset per scanline, like the number of cycles is off. I cant resolve this until the other issues are fixed.


### Status

- This needs to get filled out more
- [-] Cpu
    - [x] Instruction table
    - [x] Instructions
    - [?] Correct cpu clock timing
- [-] PPU
    - [x] OAM DMA
    - [x] Sprite tables
    - [-] Background
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

