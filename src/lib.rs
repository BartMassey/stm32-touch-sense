#![no_std]

// STM cube code: https://github.com/STMicroelectronics/STM32CubeF3.git

pub use stm32f3xx_hal::pac::TSC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TscState {
    Busy,
    Done(u16),
    Overrun,
}

pub struct TouchSense(TSC);

pub struct TouchSenseRead<'a>(&'a mut TSC);

impl TouchSense {

    pub fn new(tsc: TSC) -> TouchSense {
        // Set up control register.
        tsc.cr.write(|w| {
            unsafe {
                w.ctph()
                    .bits(0xf)
                    .ctpl()
                    .bits(0xf)
                    .ssd()
                    .bits(0x7f)
                    .sse()
                    .set_bit()
                    .sspsc()
                    .set_bit()
                    .pgpsc()
                    .bits(0x7)
                    .mcv()
                    .bits(0x6) // max pulses = 16383
                    .syncpol()
                    .clear_bit()
                    .am()
                    .clear_bit()
                    .tsce()
                    .set_bit()
            }
        });

        // On STM32F303VC Discovery Board
        // G8_IO2 is PD13, conflicts with TIM4_CH2
        // G8_IO3 is PD14, conflicts with TIM4_CH3

        // Use group pin as channel I/O.
        tsc.ioccr.write(|w| w.g8_io2().set_bit());

        // Disable group input Schmidt trigger.
        tsc.iohcr.write(|w| w.g8_io3().clear_bit());

        // Use group input as sampling capacitor.
        tsc.ioscr.write(|w| w.g8_io3().set_bit());

        Self(tsc)
    }

    pub fn into_inner(self) -> TSC {
        self.0
    }

    pub fn start<T: FnOnce()>(&mut self, discharge_wait: T) -> TouchSenseRead {
        let tsc = &mut self.0;

        // Discharge the capacitors.
        tsc.cr.write(|w| w.iodef().clear_bit());
        discharge_wait();
        tsc.cr.write(|w| w.iodef().set_bit());

        // Clear events from last acquisition.
        tsc.icr.write(|w| w.mceic().set_bit().eoaic().set_bit());

        // Enable g1 acquisition.
        tsc.iogcsr.write(|w| w.g1e().set_bit());

        // Start an acquisition.
        tsc.cr.write(|w| w.start().set_bit());

        TouchSenseRead(tsc)
    }
}

impl<'a> TouchSenseRead<'a> {

    pub fn poll(&mut self) -> TscState {
        let tsc = &mut self.0;

        let icr = tsc.icr.read();
        // Check for overrun.
        if icr.mceic().bit() {
            return TscState::Overrun;
        }

        // Poll for acquisition completion.
        if icr.eoaic().bit() {
            return TscState::Busy;
        }

        let value = tsc.iog1cr.read().cnt().bits();
        TscState::Done(value)
    }
}
