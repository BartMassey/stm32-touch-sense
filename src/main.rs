#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use aux5::{entry, Delay, DelayMs, LedArray, OutputSwitch};

#[entry]
fn main() -> ! {
    let (mut delay, mut leds): (Delay, LedArray) = aux5::init();
    let period = 100_u16;

    let mut led = |i: usize, state: bool| {
        match state {
            true => leds[i & 7].on().ok(),
            false => leds[i & 7].off().ok(),
        }
    };
    let mut wait = || delay.delay_ms(period);

    loop {
        for i in 0..8 {
            led(i, true);
            wait();
            led(i + 7, false);
            wait();
        }
    }
}
