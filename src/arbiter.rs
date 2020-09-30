use embedded_hal::{
    prelude::*,
    digital::v2::{
        InputPin,
        OutputPin,
    },
    timer::CountDown,
    //spi::FullDuplex,
    blocking::spi::Transfer,
};

use log;
//use embedded_hal::blocking::delay::DelayMs;
use core::slice::{Iter, Chunks};

use crate::buffer::Buffer;
use crate::selectable::Selectable;

use heapless::{
    String,
    consts::*,
};
use crate::delay::DelayTimer;

use embedded_time::duration::{Duration, Milliseconds};

pub struct EsWifiArbiter<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin, CD>
    where
        CD: CountDown,
        CD::Time: Duration + From<Milliseconds>,
{
    spi: SPI,
    cs: ChipSelectPin,
    ready: ReadyPin,
    wakeup: WakeUpPin,
    reset: ResetPin,
    delay: DelayTimer<CD>,
}


impl<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin, CD> EsWifiArbiter<SPI, ChipSelectPin, ReadyPin, WakeUpPin, ResetPin, CD>
    where
        SPI: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeUpPin: OutputPin,
        ResetPin: OutputPin,
        CD: CountDown,
        CD::Time: Duration + From<Milliseconds>,
{
    pub fn initialize(spi: SPI,
                      mut cs: ChipSelectPin,
                      mut ready: ReadyPin,
                      mut wakeup: WakeUpPin,
                      mut reset: ResetPin,
                      mut count_down: CD,
    ) -> Result<Self, ()> {
        EsWifiArbiter {
            cs,
            spi,
            ready,
            wakeup,
            reset,
            delay: DelayTimer::new(count_down),
        }.wait_for_prompt()
    }

    fn wait_for_prompt(mut self) -> Result<Self, ()> {
        self.wakeup();
        self.reset();

        self.cs.set_high().map_err(|_| ())?;
        self.delay.delay_ms(10);

        self.await_data_ready();

        let _cs = self.cs.select(&mut self.delay);

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

        //self.cs.set_high().map_err(|_| ())?;

        let needle = &[b'\r', b'\n', b'>', b' '];
        log::info!("look for needle {:?}", needle);

        drop(_cs);

        if !response[0..pos].starts_with(needle) {
            log::info!("failed to initialize {:?}", &response[0..pos]);
            Err(())
        } else {
            Ok(self)
        }
    }

    fn send(&mut self, command: &[u8]) -> Result<(), ()> {
        log::info!("send {:?}", command);

        self.await_data_ready();
        {
            let _cs = self.cs.select(&mut self.delay);

            for chunk in command.chunks(2) {
                let mut xfer: [u8; 2] = [0; 2];
                xfer[1] = chunk[0];
                if chunk.len() == 2 {
                    xfer[0] = chunk[1]
                } else {
                    xfer[0] = 0x0A
                }

                log::info!("transfer {:?}", xfer);
                self.spi.transfer(&mut xfer);
            }
        }
        self.await_data_ready();
        self.receive();

        Ok(())
    }

    fn receive(&mut self) -> Result<(), ()> {
        let mut response: [u8; 1024] = [0; 1024];
        let mut pos = 0;

        let _cs = self.cs.select(&mut self.delay);

        while self.ready.is_high().unwrap_or(true) {
            let mut xfer: [u8; 2] = [0x0A, 0x0A];
            self.spi.transfer(&mut xfer);
            //log::info!( "read {} {}", xfer[1] as char, xfer[0] as char);
            response[pos] = xfer[1];
            pos += 1;
            response[pos] = xfer[0];
            pos += 1;
        }
        log::info!("response {}", core::str::from_utf8(&response[0..pos]).unwrap());

        Ok(())
    }

    fn wakeup(&mut self) {
        self.wakeup.set_low();
    }

    fn reset(&mut self) {
        self.reset.set_low();
        self.delay.delay_ms(10);
        self.reset.set_high();
        self.delay.delay_ms(10);
    }

    fn await_data_ready(&mut self) {
        while self.ready.is_low().unwrap_or(false) {
            continue;
        }
    }

    pub fn get_serial_number(&mut self) -> Result<(), ()> {
        self.send(b"F0\r")?;
        Ok(())
    }


    pub fn join(&mut self, ssid: &str, password: &str) -> Result<(), ()> {
        let mut command: String<U36> = String::from("C1=");
        command.push_str(ssid);
        command.push('\r');
        self.send(command.as_bytes());

        let mut command: String<U72> = String::from("C2=");
        command.push_str(password);
        command.push('\r');
        self.send(command.as_bytes());

        let mut command: String<U72> = String::from("C3=4\r");
        //command.push_str(password);
        //command.push('\r');
        self.send(command.as_bytes());


        let mut command: String<U72> = String::from("C0\r");
        self.send(command.as_bytes());

        self.send(b"MR\r\n");

        Ok(())
    }

    pub fn resolve(&mut self, hostname: &str) -> Result<(), ()> {
        let mut command: String<U72> = String::from("D0=");
        command.push_str(hostname);
        command.push('\r');
        self.send(command.as_bytes());

        Ok(())
    }

    pub fn isr(&mut self) -> &mut ReadyPin {
        &mut self.ready
    }
}

/*
pub fn byte_swapped_and_padded<I>(cmd: &[u8]) -> I
    where I: Iterator<Item=u8>
{
    cmd.chunks(2)
        .map(|chunk| {
            if chunk.len() == 1 {
                &[b0xA0, chunk[0]]
            } else {
                &[chunk[1], chunk[0]]
            }
        })
}



 */