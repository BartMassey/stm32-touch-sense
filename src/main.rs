#![allow(unused)]
#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

// use aux5::{entry, Delay, DelayMs, LedArray, OutputSwitch};

// STM cube code: https://github.com/STMicroelectronics/STM32CubeF3.git

use cortex_m_semihosting::hio;
use core::fmt::Write;
use cortex_m::{iprint, iprintln};
use cortex_m_rt::entry;

use stm32f3_discovery::{leds::Leds, stm32f3xx_hal, switch_hal};
use switch_hal::{ActiveHigh, OutputSwitch, Switch, ToggleableOutputSwitch};

use stm32f3xx_hal::prelude::*;

use stm32f3xx_hal::{
    delay::Delay,
    gpio::{gpioe, Output, PushPull},
    hal::blocking::delay::DelayMs,
    pac,
    pac::TSC,
    pac::tsc::*,
};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum TscError {
    Timeout,
}

fn init_tsc(tsc: &mut TSC) {
    // Set up control register.
    tsc.cr.write(|w| {
        unsafe {
            w
                .ctph().bits(0xf)
                .ctpl().bits(0xf)
                .ssd().bits(0x7f)
                .sse().set_bit()
                .sspsc().set_bit()
                .pgpsc().bits(0x7)
                .mcv().bits(0x6)   // max pulses = 16383
                .syncpol().clear_bit()
                .am().clear_bit()
                .tsce().set_bit()
        }
    });

    // Use group 1 input 1 as channel I/O.
    tsc.ioccr.write(|w| w.g1_io1().set_bit());

    // Disable group 1 input 2 Schmidt trigger.
    tsc.iohcr.write(|w| w.g1_io2().clear_bit());

    // Use group 1 input 2 as sampling capacitor.
    tsc.ioscr.write(|w| w.g1_io2().set_bit());
}

fn discharge(tsc: &mut TSC, enable: bool) {
    if enable {
        tsc.cr.write(|w| w.iodef().clear_bit());
    } else {
        tsc.cr.write(|w| w.iodef().set_bit());
    }
}

fn get_value(tsc: &mut TSC) -> Result<u16, TscError> {
    // Clear events from last acquisition.
    tsc.icr.write(|w| {
        w
            .mceic().set_bit()
            .eoaic().set_bit()
    });
    
    // Enable g1 acquisition.
    tsc.iogcsr.write(|w| w.g1e().set_bit());

    // Start an acquisition.
    tsc.cr.write(|w| w.start().set_bit());

    // Poll for acquisition completion.
    while ! tsc.iogcsr.read().g1s().bit() {
        // spin
    }

    let value = tsc.iog1cr.read().cnt().bits();
    Ok(value)
}

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
    let mut led = |i: usize, state: bool| {
        match state {
            true => leds[i & 7].on().ok(),
            false => leds[i & 7].off().ok(),
        }
    };
    let mut wait = |ms| delay.delay_ms(ms);

    writeln!(stdout, "hello, world");

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
