use embedded_hal::{
    digital::v2::{
        InputPin,
        OutputPin,
    },
    //spi::FullDuplex,
    blocking::spi::Transfer,
};

use log;
use embedded_hal::blocking::delay::DelayMs;


pub struct EsWifiAdapter<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin>
    where
        SPI: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeUpPin: OutputPin,
        ResetPin: OutputPin,
{
    spi: SPI,
    cs: ChipSelectPin,
    ready: ReadyPin,
    wakeup: WakeUpPin,
    reset: ResetPin,
}

impl<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin> EsWifiAdapter<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin>
    where
        SPI: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeUpPin: OutputPin,
        ResetPin: OutputPin,
{
    pub fn new<Delay: DelayMs<u8>>(spi: SPI,
                                   mut cs: ChipSelectPin,
                                   mut ready: ReadyPin,
                                   mut wakeup: WakeUpPin,
                                   mut reset: ResetPin,
                                   delay: &mut Delay,
    ) -> Result<Self, ()> {
        EsWifiAdapter {
            cs,
            spi,
            ready,
            wakeup,
            reset,
        }.initialize(delay)
    }

    fn initialize<Delay: DelayMs<u8>>(mut self, delay: &mut Delay) -> Result<Self, ()> {
        self.wakeup.set_low().map_err(|_| ())?;

        self.reset.set_low().map_err(|_| ())?;
        delay.delay_ms(10);
        self.reset.set_high().map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;
        delay.delay_ms(10);
        loop {
            if self.ready.is_high().map_err(|_| ())? {
                break;
            }
        }
        self.cs.set_low().map_err(|_| ())?;
        delay.delay_ms(10);

        let mut response = [0 as u8; 16];
        let mut pos = 0;

        loop {
            log::info!("loop {}", pos);
            if self.ready.is_low().map_err(|_| ())? {
                break;
            }
            if pos >= response.len() {
                return Err(());
            }
            let mut chunk = [0x0A, 0x0A];
            self.spi.transfer(&mut chunk);
            log::info!("transfer {:?}", chunk);
            // reverse order going from 16 -> 2*8 bits
            if chunk[1] != 0x15 {
                response[pos] = chunk[1];
                pos += 1;
            }
            if chunk[0] != 0x15 {
                response[pos] = chunk[0];
                pos += 1;
            }
        }

        self.cs.set_high().map_err(|_| ())?;

        let needle = &[b'\r', b'\n', b'>', b' '];
        log::info!("look for needle {:?}", needle);

        if !response[0..pos].starts_with(needle) {
            log::info!("failed to initialize {:?}", &response[0..pos]);
            Err(())
        } else {
            Ok(self)
        }
    }
}
