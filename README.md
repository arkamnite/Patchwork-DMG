# Patchwork-DMG 🕹📱
An emulator for the Game Boy written in Rust.

## Why (abandon your previous attempt)? 🤔

Having initially started the AgencyGBC project (a Windows, C++ based emulator), I had made significant process before realising that I had made several errors when it came to implementing the representations of values in memory. Since starting that project, I have improved my Rust skills to a point at which I would prefer to use Rust to complete the project were I provided with an opportunity to start over.

Due to personal circumstances, I spent a lot of time away from the C++ code I had written. Looking back on it, it is in dire need of a refactor- rather than spend that time trying to remove all remnants of a GPU-rendered user interface (yes) for example, I would prefer to have a fresh go at it.

## Roadmap 🗺
You can view the current roadmap for the project here- this is the rough order in which I plan to carry out work and research, although I do not plan on "finishing" each task in sequential order. It would best be viewed as a rota on which I may rotate my efforts so that I can offer a MVP as soon as possible.

- [ ] CPU
  - [ ] Basic structure
  - [ ] Opcodes
    - [ ] 8-bit
    - [ ] 16-bit
- [ ] PPU
- [ ] Unit tests
- [ ] User interface
