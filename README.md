** NES **

Note that i'm still working on the arch of this, None of this is supposed to look clean right now
If you find yourself asking "why is this like this?", everythign is subject to change so there's no reason to invest in making something perfect yet.

The debug file is probably the "best written" one but its stil a pile of garbage.

Resources:
 - https://www.nesdev.org/wiki/Nesdev_Wiki
 - [The man himself](https://github.com/OneLoneCoder/olcNES)


This is my rust implementation of [Javid9x](https://github.com/OneLoneCoder/olcNES)s' nes emulator.
There are two goals with this: 
 - Port a `naive` cpp project to rust.
 - Optimize it once it works.

This project really is a stepping stone to later create a snes from "scratch".


### Road block
At the point of implementing the APU this project now needs to be re-written. The skeleton from a naive re-implementation is still alright, but now to propperly handle everything I will need to change how data is passed around to match a more rusty implementation.

 ### Porting a cpp project to rust.
 ---

 I have never ported a cpp project to rust before and I think its a great opportunity to learn more about both languages.


 ### Optimizing this project.
 ---
 Once we can load roms and play a game on this emulator I will start looking at optimizing it.
 I'm not starting with this because the architecture most likely will change as I add more "chips" to this projecct.
