# Patchwork-DMG 🕹📱
An emulator for the Game Boy written in Rust.

## Why (abandon your previous attempt)? 🤔

Having initially started the AgencyGBC project (a Windows, C++ based emulator), I had made significant process before realising that I had made several errors when it came to implementing the representations of values in memory. Since starting that project, I have improved my Rust skills to a point at which I would prefer to use Rust to complete the project were I provided with an opportunity to start over.

Due to personal circumstances, I spent a lot of time away from the C++ code I had written. Looking back on it, it is in dire need of a refactor- rather than spend that time trying to remove all remnants of a GPU-rendered user interface (yes) for example, I would prefer to have a fresh go at it.

## Roadmap 🗺
You can view the current roadmap for the project here- this is the rough order in which I plan to carry out work and research, although I do not plan on "finishing" each task in sequential order. It would best be viewed as a rota on which I may rotate my efforts so that I can offer a MVP as soon as possible.

- [ ] CPU
  - [ ] Basic structure
    - [x] Register pairs and associated utility functions
    - [ ] Addressing Modes and memory reading
    - [ ] Bus
  - [ ] Opcodes
    - [ ] 8-bit
    - [ ] 16-bit
- [ ] PPU
- [ ] Unit tests
- [ ] User interface

## Project Log 🌀
#### 12/11/21: CPU and related flags
This stage saw the prototype layout for the DMG CPU, including structs to represent the CPU itself,
as well as its constituent status flag register(s). The previously defined and tested `RegPair`
struct was used to implement basic register pairs. Additionally, a prototype for the LCD control register
has been implemented. 

The current testing focus is on accurate memory reading based on a given addressing mode (represented as 
an enum at the moment). Once this is complete, then work on opcode implementation can begin using closures. 

#### 10/11/21: Register Pair
I had previously laid out the blueprint for the `RegisterPair` struct which would be used to implement the DMG's 
registers, as the name implies. Specifically however, this struct will ideally take a closure in order to manipulate its fields-
two `u8` variables, one for each register- whilst also offering necessary functionality such as conversion between decimal and BCD
representations in binary. Therefore, I am designing this struct in a way that it can return an appropriate `Result` based on whether there
were any issues or any warnings, such as overflows or carries. This would ideally help me save time when implementing opcodes as I can pattern match this result
then set the appropriate flags within the CPU.

Currently, the decimal-to-BCD function has been completed as well as tested with a small unit test. However, it uses
an unideal method of concatenating two strings and then parsing this as a `u8` which has a slightly unnecessary memory
footprint.

