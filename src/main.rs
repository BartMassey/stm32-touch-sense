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

fn initialize(tsc: &mut TSC) {
    // enable TSC
    tsc.cr.write(|w| w.sse().set_bit());

    // set all functions

    unsafe { tsc.cr.write(|w| w
                          .ctph().bits(2)
                          .ctpl().bits(2)
                          .ssd().bits(1)
                          .sse().set_bit()
                          .sspsc().set_bit()
                          .pgpsc().bits(1)
                          .mcv().bits(2)
                          .syncpol().clear_bit()
                          .am().set_bit())
    }
}

fn discharge(tsc: &mut TSC, enable: bool) {

    if enable {
        tsc.cr.write(|w| w.iodef().clear_bit());
    } else {
        tsc.cr.write(|w| w.iodef().set_bit());
    }
}

fn ioconfig() {
    todo!()
}

fn start() {
    todo!()
}


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Error {
    Timeout,
    Invalid,
    MiscError,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum TscState {
    Reset,
    Ready,
    Busy,
    Error,
}

fn getstate() -> Result<TscState,Error> {
    todo!()
}

fn poll() -> Result<(),Error> {
    todo!()
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum TscGroupStatus {
    Ongoing,
    Completed,
}

fn status() -> Result<TscGroupStatus,Error> {
    todo!()
}

fn get_value() -> u32 {
    todo!()
}

type LedArray = [Switch<gpioe::PEx<Output<PushPull>>, ActiveHigh>; 8];

fn init() -> (Delay, LedArray, hio::HStdout, TSC) {
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

    return (delay, leds.into_array(), stdout, tsc);
}

#[entry]
fn main() -> ! {
    let (mut delay, mut leds, mut stdout, mut tsc) = init();
    let period = 100_u16;

    let mut led = |i: usize, state: bool| {
        match state {
            true => leds[i & 7].on().ok(),
            false => leds[i & 7].off().ok(),
        }
    };
    let mut wait = || delay.delay_ms(period);

    writeln!(stdout, "hello, world");

    initialize(&mut tsc);

    loop {
        wait();
        discharge(&mut tsc, true);
        wait();
        discharge(&mut tsc, false);
        wait();
        ioconfig();
        start();
        poll();

        let s = match status() {
            Err(e) => {
                writeln!(stdout, "error: {:?}", e).unwrap();
                continue;
            }
            Ok(v) => v,
        };
        if (s != TscGroupStatus::Completed) {
            writeln!(stdout, "status: {:?}", s).unwrap();
            continue;
        }

        let value: u32 = get_value();
        writeln!(stdout, "value: {}", value).unwrap();
    }
}
