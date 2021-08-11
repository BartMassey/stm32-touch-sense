#![no_std]

#[cfg(feature="stm32f3xx")]
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
                w
                    // Charge transfer pulse high (clocks)
                    .ctph()
                    .bits(0x0)

                    // Charge transfer pulse low (clocks)
                    .ctpl()
                    .bits(0x4)

                    // Spread spectrum (see manual)
                    .ssd()
                    .bits(0x7f)

                    // Spread spectrum enable
                    .sse()
                    .clear_bit()

                    // Spread spectrum prescaler
                    .sspsc()
                    .clear_bit()

                    // Pulse generator prescaler (clock division)
                    .pgpsc()
                    .bits(0x0)

                    // Max count value (counts)
                    .mcv()
                    .bits(0x6) // max pulses = 16383

                    // I/O default mode
                    .iodef()
                    .clear_bit()

                    // Sync pin polarity
                    .syncpol()
                    .clear_bit()

                    // Acq mode
                    .am()
                    .clear_bit()

                    // Start acq
                    .start()
                    .clear_bit()

                    // TSC Enable
                    .tsce()
                    .set_bit()
            }
        });

        // On STM32F303VC Discovery Board
        // G8_IO2 is PD13, conflicts with TIM4_CH2
        // G8_IO3 is PD14, conflicts with TIM4_CH3

        // Use group pin as channel I/O.
        tsc.ioccr.modify(|_, w| w.g8_io2().set_bit());

        // Disable group pin Schmidt trigger.
        tsc.iohcr.modify(|_, w| w.g8_io2().clear_bit());

        // Disable group input Schmidt trigger.
        tsc.iohcr.modify(|_, w| w.g8_io3().clear_bit());

        // Disable group pin analog switch.
        tsc.ioascr.modify(|_, w| w.g8_io2().set_bit());

        // Disable group input analog switch.
        tsc.ioascr.modify(|_, w| w.g8_io3().set_bit());

        // Use group input as sampling capacitor.
        tsc.ioscr.modify(|_, w| w.g8_io3().set_bit());

        Self(tsc)
    }

    pub fn into_inner(self) -> TSC {
        self.0
    }

    pub fn start<T: FnOnce()>(&mut self, discharge_wait: T) -> TouchSenseRead {
        let tsc = &mut self.0;

        // Discharge the capacitors.
        tsc.cr.modify(|_, w| w.iodef().clear_bit());
        discharge_wait();
        tsc.cr.modify(|_, w| w.iodef().set_bit());

        // Clear events from last acquisition.
        tsc.icr.modify(|_, w| w.mceic().set_bit().eoaic().set_bit());

        // Enable group acquisition.
        tsc.iogcsr.modify(|_, w| w.g8e().set_bit());

        // Start an acquisition.
        tsc.cr.modify(|_, w| w.start().set_bit());

        TouchSenseRead(tsc)
    }
}

impl<'a> TouchSenseRead<'a> {

    pub fn poll(&mut self) -> TscState {
        let tsc = &mut self.0;

        let isr = tsc.isr.read();
        // Check for overrun.
        if isr.mcef().bit() {
            return TscState::Overrun;
        }

        // Poll for acquisition completion.
        if isr.eoaf().bit() {
            let value = tsc.iog8cr.read().cnt().bits();
            return TscState::Done(value);
        }

        TscState::Busy
    }
}
