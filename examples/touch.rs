#![no_std]
#![no_main]

use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;

use stm32f3_discovery::{leds::Leds, stm32f3xx_hal, switch_hal};
use switch_hal::{ActiveHigh, OutputSwitch, Switch};

use stm32f3xx_hal::{
    prelude::*,
    delay::Delay,
    gpio::{gpioe, Output, PushPull},
    pac,
};

use touch_sense::*;

type LedArray = [Switch<gpioe::PEx<Output<PushPull>>, ActiveHigh>; 8];

fn init_periphs() -> (Delay, LedArray, hio::HStdout, TouchSense) {

    let device_periphs = pac::Peripherals::take().unwrap();
    let mut rcc = device_periphs.RCC.constrain();

    let core_periphs = cortex_m::Peripherals::take().unwrap();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let delay = Delay::new(core_periphs.SYST, clocks);

    let mut stdout = hio::hstdout().unwrap();

    // initialize tsc
    unsafe { &(*pac::RCC::ptr()).ahbenr.write(|w| w.tscen().set_bit()) };
    let mut gpiod = device_periphs.GPIOD.split(&mut rcc.ahb);
    let _pd13 = gpiod.pd13.into_af3_push_pull(
        &mut gpiod.moder,
        &mut gpiod.otyper,
        &mut gpiod.afrh,
    );
    let _pd14 = gpiod.pd14.into_af3_push_pull(
        &mut gpiod.moder,
        &mut gpiod.otyper,
        &mut gpiod.afrh,
    );
    let tsc = device_periphs.TSC;
    let touch_sense = TouchSense::new(tsc);

    // initialize user leds
    let mut gpioe = device_periphs.GPIOE.split(&mut rcc.ahb);
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

    (delay, leds.into_array(), stdout, touch_sense)
}

#[entry]
fn main() -> ! {
    let (mut delay, mut leds, mut stdout, mut touch_sense) = init_periphs();
    let mut led = |i: usize, state: bool| match state {
        true => leds[i & 7].on().ok(),
        false => leds[i & 7].off().ok(),
    };
    let mut wait = |ms| delay.delay_ms(ms);

    loop {
        writeln!(stdout, "starting acq").unwrap();
        let mut sensor = touch_sense.start(|| wait(10u32));
        
        loop {
            led(0, true);
            match sensor.poll() {
                TscState::Busy => (),
                TscState::Overrun => {
                    writeln!(stdout, "overrun").unwrap();
                    break;
                }
                TscState::Done(value) => {
                    writeln!(stdout, "value: {}", value).unwrap();
                    break;
                }
            }
        }
        writeln!(stdout, "ending acq").unwrap();
        led(0, false);
        wait(1000u32);
    }
}
