use crate::*;

struct Ss {
    ssd: u8,
    sspsc: bool,
}

struct Timing {
    ctph: u8,
    ctpl: u8,
    pgpsc: u8,
}

pub struct TouchSenseConfig {
    timing: Timing,
    ss: Option<Ss>,
    mcv: u8,
    iodef: bool,
    syncpol: bool,
    am: bool,
}

pub struct TscConfigError;

fn tsc_config_field<T, const N: u32>(val: T) -> Result<u8, TscConfigError>
    where T: Into<u32>
{
    let v: u32 = val.into();
    if v < (1 << N) {
        Ok(v as u8)
    } else {
        Err(TscConfigError)
    }
}

impl Default for TouchSenseConfig {
    fn default() -> Self {
        Self {
            timing: Timing {
                ctph: 1,
                ctpl: 2,
                pgpsc: 0,
            },
            ss: None,
            mcv: 6,
            iodef: true,
            syncpol: false,
            am: false,
        }
    }
}

impl TouchSenseConfig {

    pub fn set_timing<T, U, V>(&mut self, ctph: T, ctpl: U, pgpsc: V)
                           -> Result<&mut Self, TscConfigError>
        where T: Into<u32>, U: Into<u32>, V: Into<u32>
    {
        let ctpl: u32 = ctpl.into();
        let pgpsc: u32 = pgpsc.into();
        if ctpl + pgpsc < 2 {
            return Err(TscConfigError);
        }
        self.timing.ctph = tsc_config_field::<_, 4>(ctph)?;
        self.timing.ctpl = tsc_config_field::<_, 4>(ctpl)?;
        self.timing.pgpsc = tsc_config_field::<_, 3>(pgpsc)?;
        Ok(self)
    }

    pub fn set_spread_spectrum<T>(&mut self, ssd: T, sspsc: bool)
                                  -> Result<&mut Self, TscConfigError>
        where T: Into<u32>
    {
        self.ss = Some( Ss {
            ssd: tsc_config_field::<_, 6>(ssd)?,
            sspsc
        });
        Ok(self)
    }

    pub fn set_max_acq_count<T>(&mut self, mcv: T)
                                -> Result<&mut Self, TscConfigError>
        where T: Into<u32>
    {
        let mcv: u32 = mcv.into();
        if mcv >= 7 {
            return Err(TscConfigError);
        }
        self.mcv = mcv as u8;
        Ok(self)
    }

    pub fn set_io_default_mode(&mut self, iodef: bool) -> &mut Self {
        self.iodef = iodef;
        self
    }

    pub fn set_sync_pin_polarity(&mut self, syncpol: bool) -> &mut Self {
        self.syncpol = syncpol;
        self
    }

    pub fn set_sync_acq_mode(&mut self, am: bool) -> &mut Self {
        self.am = am;
        self
    }


    pub fn config(self, tsc: TSC) -> TouchSense {
        // Set up control register.
        tsc.cr.write(|w| {
            unsafe {
                w
                    // Charge transfer pulse high (clocks)
                    .ctph()
                    .bits(self.timing.ctph)

                    // Charge transfer pulse low (clocks)
                    .ctpl()
                    .bits(self.timing.ctpl)

                    // Spread spectrum disable
                    .sse()
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

        if let Some(ss) = self.ss {
            tsc.cr.modify(|_, w| {
                unsafe {
                    let w = w
                        // Spread spectrum enable
                        .sse()
                        .set_bit()

                        // Spread spectrum deviation
                        .ssd()
                        .bits(ss.ssd)

                        // Spread spectrum prescaler
                        .sspsc()
                        .clear_bit();

                    if ss.sspsc {
                        // Spread spectrum prescaler
                        w.sspsc().set_bit()
                    } else {
                        w
                    }
                }
            });
        } 

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

        TouchSense(tsc)
    }
}
