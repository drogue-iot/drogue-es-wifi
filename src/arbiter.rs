use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use drogue_embedded_timer::Delay;
use heapless::{
    consts::*,
    String,
    spsc::{Consumer, Producer}
};

use crate::protocol::{Request, Response, JoinInfo};
use crate::chip_select::ChipSelect;
use embedded_time::duration::Milliseconds;
use crate::ready::Ready;

enum State {
    Uninitialized,
    Ready,
}

pub struct Arbiter<'clock, 'q, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    spi: Spi,
    cs: ChipSelect<'clock, ChipSelectPin, Clock>,
    ready: Ready<ReadyPin>,
    wakeup: WakeupPin,
    reset: ResetPin,
    delay: Delay<'clock, Clock>,
    consumer: Consumer<'q, Request, U1>,
    producer: Producer<'q, Response, U1>,
    state: State,
}

impl<'clock, 'q, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock> Arbiter<'clock, 'q, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    pub fn new(spi: Spi,
               cs: ChipSelectPin,
               ready: ReadyPin,
               wakeup: WakeupPin,
               reset: ResetPin,
               delay: Delay<'clock, Clock>,
               consumer: Consumer<'q, Request, U1>,
               producer: Producer<'q, Response, U1>,
    ) -> Self {
        Self {
            spi,
            cs: ChipSelect::new(cs, delay.clone()),
            ready: Ready::new(ready),
            wakeup,
            reset,
            delay,
            consumer,
            producer,
            state: State::Uninitialized,
        }
    }

    fn initialize(&mut self) -> Result<(),()>{
        self.wakeup();
        self.reset();

        log::info!("await ready");
        self.await_data_ready();
        log::info!("ready");

        let _cs = self.cs.select();

        let mut response = [0 as u8; 16];
        let mut pos = 0;

        loop {
            log::info!("loop {}", pos);
            if ! self.ready.is_ready() {
                break;
            }
            if pos >= response.len() {
                return Err(())
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

        let needle = &[b'\r', b'\n', b'>', b' '];
        log::info!("look for needle {:?} {}", needle, pos);

        if !response[0..pos].starts_with(needle) {
            log::info!("failed to initialize {:?}", &response[0..pos]);
            Err(())
        } else {
            self.state = State::Ready;
            Ok(())
        }
    }

    fn process_requests(&mut self) {
        if let Some(request) = self.consumer.dequeue() {
            log::info!("handle request: {:?}", request);

            match request {
                Request::Join(ref join_info) => {
                    self.join(join_info);
                },
            }
        }
    }

    pub fn isr(&mut self) -> Result<(),()>{
        if matches!(self.state, State::Uninitialized) {
            self.initialize()?;
        }

        self.process_requests();

        Ok(())
    }

    fn wakeup(&mut self) {
        self.wakeup.set_low();
    }

    fn reset(&mut self) {
        self.reset.set_low();
        self.delay.delay(Milliseconds(10u32));
        self.reset.set_high();
        self.delay.delay(Milliseconds(10u32));
    }

    fn await_data_ready(&mut self) {
        while ! self.ready.is_ready() {
            continue;
        }
    }

    fn send(&mut self, command: &[u8], response: &mut [u8]) -> Result<usize, ()> {
        log::info!("send {:?}", command);

        self.await_data_ready();
        {
            let _cs = self.cs.select();

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
        self.receive(response)
    }

    fn receive(&mut self, response: &mut [u8]) -> Result<usize, ()> {
        let mut pos = 0;

        let _cs = self.cs.select();

        while self.ready.is_ready() {
            let mut xfer: [u8; 2] = [0x0A, 0x0A];
            self.spi.transfer(&mut xfer);
            log::info!( "read {} {}", xfer[1] as char, xfer[0] as char);
            response[pos] = xfer[1];
            pos += 1;
            response[pos] = xfer[0];
            pos += 1;
        }
        log::info!("response {}", core::str::from_utf8(&response[0..pos]).unwrap());

        Ok(pos)
    }

    // ------------------------------------------------------------------------
    // Request handling
    // ------------------------------------------------------------------------

    fn join(&mut self, join_info: &JoinInfo) {
        log::info!("JOIN {:?}", join_info);
        match join_info {
            JoinInfo::Open => {

            },
            JoinInfo::Wep { ssid, password } => {
                let mut response = [0u8; 1024];
                let mut command: String<U36> = String::from("C1=");
                command.push_str(ssid);
                command.push('\r');
                self.send(command.as_bytes(), &mut response);

                let mut command: String<U72> = String::from("C2=");
                command.push_str(password);
                command.push('\r');
                self.send(command.as_bytes(), &mut response);

                let mut command: String<U72> = String::from("C3=4\r");
                //command.push_str(password);
                //command.push('\r');
                self.send(command.as_bytes(), &mut response);

                let mut command: String<U72> = String::from("C0\r");
                if let Ok( len ) = self.send(command.as_bytes(), &mut response) {
                    if let Ok( (_,response) )= crate::parser::join_response( &response[0..len]) {
                        self.producer.enqueue(response );
                    } else {
                        log::info!("failed to parse {:?}", &response[0..4]);
                    }
                } else {
                    log::info!("failed to send")
                }

            },
        }
    }

}