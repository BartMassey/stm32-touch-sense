#![no_std]
#![no_main]

#[cfg(feature="touch_debug")]
mod touch_debug {
    pub(crate) use core::fmt::Write;
    pub(crate) use cortex_m_semihosting::hio;
    // logs messages to the host stderr; requires a debugger
    pub(crate) use panic_semihosting as _;
}
#[cfg(feature="touch_debug")]
use touch_debug::*;

#[cfg(not(feature="touch_debug"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use cortex_m_rt::entry;
use stm32f3_discovery::{leds::Leds, stm32f3xx_hal, switch_hal};
use switch_hal::{ActiveHigh, OutputSwitch, Switch};

use stm32f3xx_hal::{
    prelude::*,
    delay::Delay,
    gpio::{gpioe, Output, PushPull},
    pac,
};

use stm32_touch_sense::*;

type LedArray = [Switch<gpioe::PEx<Output<PushPull>>, ActiveHigh>; 8];

fn init_periphs() -> (Delay, LedArray, TouchSense) {

    let device_periphs = pac::Peripherals::take().unwrap();
    let mut rcc = device_periphs.RCC.constrain();

    let core_periphs = cortex_m::Peripherals::take().unwrap();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = rcc.cfgr
        // The sysclk is equivalent to the core clock
        .sysclk(48.MHz())
        // Set the frequency for the AHB bus,
        // which the root of every following clock peripheral
        .hclk(48.MHz())
        // Freeze / apply the configuration and setup all clocks
        .freeze(&mut flash.acr);

    let delay = Delay::new(core_periphs.SYST, clocks);

    // initialize tsc
    let ahbenr = unsafe { &(*pac::RCC::ptr()).ahbenr };
    ahbenr.write(|w| w.tscen().set_bit());
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
    let touch_sense = TouchSenseConfig::default().config(tsc);

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

    (delay, leds.into_array(), touch_sense)
}

#[entry]
fn main() -> ! {
    #[cfg(feature="touch_debug")]
    let mut stdout = hio::hstdout().unwrap();
    let (mut delay, mut leds, mut touch_sense) = init_periphs();
    let mut led = |i: usize, state: bool| match state {
        true => leds[i & 7].on().ok(),
        false => leds[i & 7].off().ok(),
    };
    let mut wait = |us| delay.delay_us(us);

    loop {
        #[cfg(feature="touch_debug")]
        writeln!(stdout, "starting acq").unwrap();
        let mut sensor = touch_sense.start(|| wait(100u32));
        
        loop {
            led(0, true);
            led(2, false);
            match sensor.poll() {
                TscState::Busy => (),
                TscState::Overrun => {
                    #[cfg(feature="touch_debug")]
                    writeln!(stdout, "overrun").unwrap();
                    led(2, true);
                    break;
                }
                TscState::Done(value) => {
                    #[cfg(feature="touch_debug")]
                    writeln!(stdout, "value: {}", value).unwrap();
                    led(1, value <= 60);
                    break;
                }
            }
        }
        #[cfg(feature="touch_debug")]
        writeln!(stdout, "ending acq").unwrap();
        led(0, false);
        #[cfg(feature="touch_debug")]
        wait(1_000_000u32);
    }
}
