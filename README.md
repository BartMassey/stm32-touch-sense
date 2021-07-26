# STM32 Touch Sense
Bart Massey 2021

This codebase comprises:

* A `touch_sense` library crate providing a higher-level
  interface to the Touch Sense Controller (TSC) peripheral
  provided by many STM32-family System-on-Chip devices (ST
  Microelectronics 32-bit ARM SoCs).

* An example `touch` application showing the use of the
  library with the STM32F303VC Discovery Board.

* Infrastructure needed to run and debug the example
  application.

The intent is to provide a library that fits cleanly into
the Embedded Rust ecosystem and works across the full range
of STM32 devices with this peripheral.

## Work In Progress

This library is still at an extremely preliminary
stage. Much is hard-coded that should not be, and
development and debugging is still in progress. Things are
not at all ready for use yet, so *caveat emptor*.

## Running The Touch Example

* Hook up an STM32F303VC Discovery Board to your Linux Box.

* Connect a 47pF sampling capacitor between pins PD14 and
  ground on the Disco Board. Connect a touchwire to PD13 via
  1Kohm static protection resistor.

* Start `openocd` at the project root from a separate terminal.

* `cargo run --release --example touch`

* Watch the count values from the acquisitions.

## Acknowledgements

Thanks much to Keith Packard, who helped me to figure stuff
out, wrote the original code with me, and helped me fix and
debug the first working version.

The `cortex-m-quickstart` template was really helpful here,
as was the Embedded Rust Book and the Embedded Discovery
Book.

This project was based on a bunch of STM documentation.  The
best place to start is probably *Getting started with touch
sensing control on STM32 microcontrollers* —
[STM AN5105](https://www.st.com/resource/en/application_note/dm00445657-getting-started-with-touch-sensing-control-on-stm32-microcontrollers-stmicroelectronics.pdf).
This document lists other Application Notes and materials
that are helpful in understanding the operation of the TSC.

The code from STM's
[STM32CubeF3](https://github.com/STMicroelectronics/STM32CubeF3.git)
C library was studied in the development of this software;
specifically
`Drivers/STM32F3xx_HAL_Driver/Src/stm32f3xx_hal_tsc.c` which
is available under the 3-clause BSD License.

## License

This work is made available under the "3-clause ('new') BSD
License". Please see the file `LICENSE` in this distribution
for license terms.
