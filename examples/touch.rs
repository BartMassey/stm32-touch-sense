#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

// use aux5::{entry, Delay, DelayMs, LedArray, OutputSwitch};

// STM cube code: https://github.com/STMicroelectronics/STM32CubeF3.git

use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;

use stm32f3_discovery::{leds::Leds, stm32f3xx_hal, switch_hal};
use switch_hal::{ActiveHigh, OutputSwitch, Switch};

use stm32f3xx_hal::prelude::*;

use stm32f3xx_hal::{
    delay::Delay,
    gpio::{gpioe, Output, PushPull},
    pac,
    pac::TSC,
};

type LedArray = [Switch<gpioe::PEx<Output<PushPull>>, ActiveHigh>; 8];

fn init_periphs() -> (Delay, LedArray, hio::HStdout, TSC) {
    let device_periphs = pac::Peripherals::take().unwrap();
    let mut reset_and_clock_control = device_periphs.RCC.constrain();

    let core_periphs = cortex_m::Peripherals::take().unwrap();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = reset_and_clock_control.cfgr.freeze(&mut flash.acr);
    let delay = Delay::new(core_periphs.SYST, clocks);
    let tsc = device_periphs.TSC;

    // initialize user leds
    let mut gpioe = device_periphs.GPIOE.split(&mut reset_and_clock_control.ahb);
    let leds = Leds::new(
        gpioe.pe8,
        gpioe.pe9,
        gpioe.pe10,
        gpioe.pe11,
        gpioe.pe12,
        gpioe.pe13,
        gpioe.pe14,
        gpioe.pe15,
        &mut gpioe.moder,
        &mut gpioe.otyper,
    );

    let stdout = hio::hstdout().unwrap();

    (delay, leds.into_array(), stdout, tsc)
}

#[entry]
fn main() -> ! {
    let (mut delay, mut leds, mut stdout, mut tsc) = init_periphs();
    let mut led = |i: usize, state: bool| match state {
        true => leds[i & 7].on().ok(),
        false => leds[i & 7].off().ok(),
    };
    let mut wait = |ms| delay.delay_ms(ms);

    writeln!(stdout, "starting acq").unwrap();
    led(0, true);

    init_tsc(&mut tsc);

    loop {
        discharge(&mut tsc, true);
        wait(10u32);
        discharge(&mut tsc, false);
        let value = match get_value(&mut tsc) {
            Err(e) => {
                writeln!(stdout, "error: {:?}", e).unwrap();
                continue;
            }
            Ok(v) => v,
        };
        writeln!(stdout, "value: {}", value).unwrap();
    }
}
