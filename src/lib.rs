#![no_std]

pub mod config;
pub use config::*;

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
