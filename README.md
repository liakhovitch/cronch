# Crönch!
Not-entirely-sane digital delay pedal madness

**Note: This project is not finished. PCB contains undocumented errors and code may not compile.**

## What is this?
Crönch is a hardware audio effects processor unit, inspired by the Sonic Charge Permut8 software plugin. For a more detailed explanation, see [the proposal document](proposal/proposal.pdf).

Some key points:
- Hardware designed in KiCAD
- Firmware written in Rust
- Based on the RP2040 microcontroller and TI TLV320AIC3254 audio CODEC
- Runs at a 192KHz audio sampling rate

## Repository map
[Schematic](pcb/effectpedal/fabrication/schematic.pdf)

[Firmware](firmware/src)

[Original proposal](proposal/proposal.pdf)

## Todo
- [x] Preliminary design
- [X] Choosing components
- [X] Electrical design (schematic)
- [X] PCB design
- [X] Hardware assembly
- [X] Basic hardware test (MCU can read/write from each external peripheral)
- [X] Full hardware test (MCU can access all features of each external peripheral)
- [X] Multicore concurrency primitives
- [X] Front panel UI
- [ ] All hardware driver code complete
- [ ] Basic functionality (Sample incoming audio, write it to the buffer, then immediately send it back to the CODEC)
- [ ] All operators implemented
- [ ] All audio knobs (mix, feedback, gain, etc) implemented
- [ ] Variable clock rate

## Images
