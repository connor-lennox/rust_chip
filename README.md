# rust_chip

A CHIP-8 emulator written in Rust. Uses SDL-2 for graphics + key events.

The code is split into several files, mirroring how the COSMAC-VIP would actually run CHIP-8 programs.

## cpu.rs

The CPU is represented by a struct containing the memory (array of `u8` bytes), registers (another `u8` array), various callstack related elements, a couple of timers, and a reference to a Keypad and Display struct.

Opcodes are implemented via a large `match` statement. Rust is wonderful for this as it can handle situations where a nested statement would be required in other languages (for example, the `0x8xxx` opcodes are often implemented with a nested switch-case in C). 

The CPU of CHIP-8 interpreters has no defined clock rate (as the CHIP-8 was never a "real" hardware device, just an interpreter for other computers) so the `main.rs` file has a const field setting the clock speed to 700 operations per second. This is a "pretty good" value for most games, although some require it to be higher/lower.


## display.rs

The display is another struct, containing only a single (flattened) array of boolean values: a value of `true` represents that the pixel is "on", and `false` represents an "off" pixel. CHIP-8 is designed to work with monochrome graphics, so this is sufficient.

An interesting quirk of the CHIP-8 is that sprites don't set bits to "on", they actually *flip* the bits that draw the sprite. This behavior is then extended to provide a pixel-perfect collision checker: if a sprite is drawn and any pixel was flipped to "off", it is known that this newly drawn sprite is intersecting something else that's already drawn to screen.


## keypad.rs

This struct just holds the current state of the keypad (the COSMAC-VIP had 16 keys labelled 0-F) but also holds onto the most recently released key and an indicator of whether or not the key was released this cycle. Only one instruction takes advantage of this (`0xFx0A`) but it's important as this instruction blocks until the next key is *released* (note: not when the next key is pressed). As such, calling this instruction while the keypad has no freshly released key will just back up the PC so that the instruction is called again.


## Included Games

The cartridges in this repository (located in `./carts/*`) were not made by me, but serve as good tests of the CHIP-8 and its functionality. The files `bc_test.ch8`, `test_opcode.ch8`, and `ibm_logo.ch8` all test certain opcodes and were used during development to ensure the emulator worked as expected. `br8kout.ch8` is a Breakout clone for the CHIP-8 which served as the test ROM to make sure games were functional.


# Usage

Using this repo requires that you have the Rust compiler installed. To run the emulator, navigate to the root directory and run:

```cargo run```

This will run the default cartridge (`./test_opcode.ch8`). This isn't a particularly fun default cart, but it should let you know that everything is functional. The main function takes a filepath as an argument, and if provided will attempt to load that cartridge instead of the default. For example:

```cargo run ./carts/br8kout.ch8```

will run the Breakout game. Other games are available online, if you want to try out some more.