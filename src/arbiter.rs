use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use embedded_time::duration::Milliseconds;
use drogue_embedded_timer::Delay;
use heapless::{consts::*, String, spsc::{Consumer, Producer}, ArrayLength};

use core::fmt::Write;

use crate::protocol::{Response, ConnectInfo, ConnectionType, WriteInfo};
use crate::chip_select::ChipSelect;
use crate::ready::Ready;
use nom::InputIter;
use crate::adapter::{AdapterError, JoinError, JoinInfo};
use crate::parser;
use crate::parser::JoinResponse;
use nom::error::ErrorKind;

macro_rules! command {
    ($size:tt, $($arg:tt)*) => ({
        //let mut c = String::new();
        //c
        let mut c = String::<$size>::new();
        write!(c, $($arg)*);
        c.push_str("\r");
        c
    })
}

enum State {
    Uninitialized,
    Ready,
}


#[derive(Debug)]
enum SpiError {
    ReadError,
    WriteError,
}


pub struct Arbiter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
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
    state: State,
}

impl<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock> Arbiter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
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
    ) -> Self {
        Self {
            spi,
            cs: ChipSelect::new(cs, delay.clone()),
            ready: Ready::new(ready),
            wakeup,
            reset,
            delay,
            state: State::Uninitialized,
        }
    }

    fn initialize(&mut self) -> Result<(), ()> {
        self.wakeup();
        self.reset();

        //log::info!("await ready");
        self.await_data_ready();
        //log::info!("ready");

        let _cs = self.cs.select();

        let mut response = [0 as u8; 16];
        let mut pos = 0;

        loop {
            //log::info!("loop {}", pos);
            if !self.ready.is_ready() {
                break;
            }
            if pos >= response.len() {
                return Err(());
            }
            let mut chunk = [0x0A, 0x0A];
            self.spi.transfer(&mut chunk);
            //log::info!("transfer {:?}", chunk);
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
        //log::info!("look for needle {:?} {}", needle, pos);

        drop(_cs);

        if !response[0..pos].starts_with(needle) {
            log::info!("failed to initialize {:?}", &response[0..pos]);
            Err(())
        } else {
            // disable verbosity
            self.send_string(&command!(U8, "MT=1"), &mut response);
            self.state = State::Ready;
            log::info!("eS-WiFi adapter is ready");
            Ok(())
        }
    }

    fn process_backlog(&mut self) {
        if matches!(self.state, State::Uninitialized) {
            self.initialize();
        }
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
        while !self.ready.is_ready() {
            continue;
        }
    }

    fn send_string<'a, N: ArrayLength<u8>>(&mut self, command: &String<N>, response: &'a mut [u8]) -> Result<&'a [u8], SpiError> {
        self.send(command.as_bytes(), response)
    }

    fn send<'a>(&mut self, command: &[u8], response: &'a mut [u8]) -> Result<&'a [u8], SpiError> {
        log::info!("send {:?}", core::str::from_utf8(command).unwrap());

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

                let result = self.spi.transfer(&mut xfer);
                if !result.is_ok() {
                    return Err(SpiError::WriteError);
                }
            }
        }
        self.await_data_ready();
        self.receive(response)
    }

    fn receive<'a>(&mut self, response: &'a mut [u8]) -> Result<&'a [u8], SpiError> {
        let mut pos = 0;

        let _cs = self.cs.select();

        while self.ready.is_ready() {
            let mut xfer: [u8; 2] = [0x0A, 0x0A];
            let result = self.spi.transfer(&mut xfer);
            if !result.is_ok() {
                return Err(SpiError::ReadError);
            }
            //log::info!( "read {} {}", xfer[1] as char, xfer[0] as char);
            response[pos] = xfer[1];
            pos += 1;
            if xfer[0] != 0x15 {
                response[pos] = xfer[0];
                pos += 1;
            }
        }
        log::info!("response {}", core::str::from_utf8(&response[0..pos]).unwrap());
        //Ok(pos)
        Ok(&mut response[0..pos])
    }

    // ------------------------------------------------------------------------
    // Request handling
    // ------------------------------------------------------------------------

    pub(crate) fn join(&mut self, join_info: &JoinInfo) -> Result<(), JoinError> {
        self.process_backlog();
        match join_info {
            JoinInfo::Open => {
                Ok(())
            }
            JoinInfo::Wep { ssid, password } => {
                let mut response = [0u8; 1024];

                self.send_string(
                    &command!(U36, "CB=2"),
                    &mut response).map_err(|_| JoinError::InvalidSsid)?;

                self.send_string(
                    &command!(U36, "C1={}", ssid),
                    &mut response).map_err(|_| JoinError::InvalidSsid)?;

                self.send_string(
                    &command!(U72, "C2={}", password),
                    &mut response).map_err(|_| JoinError::InvalidPassword)?;

                self.send_string(
                    &command!(U8, "C3=4"),
                    &mut response).map_err(|_| JoinError::Unknown)?;


                let response = self.send_string(&command!(U4, "C0"), &mut response).map_err(|_| JoinError::Unknown)?;

                log::info!( "[[{}]]", core::str::from_utf8(&response).unwrap());

                let parse_result = parser::join_response(&response);

                log::info!("response for JOIN {:?}", parse_result);

                match parse_result {
                    Ok((_, response)) => {
                        match response {
                            JoinResponse::Ok => {
                                Ok(())
                            }
                            JoinResponse::JoinError => {
                                Err(JoinError::UnableToAssociate)
                            }
                        }
                    }
                    Err(e) => {
                        log::info!( "{:?}", &response);
                        Err(JoinError::UnableToAssociate)
                    }
                }
                /*
                if parse_result.is_ok() {
                    let (_, response) = parse_result.unwrap();
                    match response {
                        JoinResponse::Ok => {
                            Ok(())
                        }
                        JoinResponse::JoinError => {
                            Err(JoinError::UnableToAssociate)
                        }
                    }
                } else {
                    Err(JoinError::UnableToAssociate)
                }

                 */
            }
        }

        //Err(JoinError::Unknown)
    }

    pub(crate) fn connect(&mut self, connect_info: &ConnectInfo) -> Result<Response, ()> {
        log::info!("CONNECT {:?}", connect_info);

        let mut response = [0u8; 1024];
        let mut command: String<U8> = String::from("P0=");
        write!(command, "{}\r", connect_info.socket_num).unwrap();
        self.send(command.as_bytes(), &mut response);

        match connect_info.connection_type {
            ConnectionType::Tcp => {
                self.send("P1=0\r".as_bytes(), &mut response);
            }
            ConnectionType::Udp => {
                self.send("P1=1\r".as_bytes(), &mut response);
            }
        }

        let mut command: String<U32> = String::from("P3=");
        write!(command, "{}\r", connect_info.remote.addr().ip());
        self.send(command.as_bytes(), &mut response);

        let mut command: String<U32> = String::from("P4=");
        write!(command, "{}\r", connect_info.remote.port());
        self.send(command.as_bytes(), &mut response);

        if let Ok(reply) = self.send("P6=1\r".as_bytes(), &mut response) {
            if let Ok((_, response)) = parser::connect_response(&reply) {
                return Ok(response);
            } else {
                log::info!("failed to parse {:?}", core::str::from_utf8(&reply).unwrap());
            }
        }
        Err(())
    }

    pub(crate) fn write(&mut self, write_info: &WriteInfo) -> Result<usize, ()> {
        self.process_backlog();
        log::info!("WRITE {:?}", write_info);

        let mut len = write_info.data.len();
        if len > 1046 {
            len = 1046
        }

        let mut response = [0u8; 1024];

        let command = command!(U8, "P0={}", write_info.socket_num);
        self.send(command.as_bytes(), &mut response);

        let mut command: String<U16> = String::from("S1=");
        write!(command, "{}\r", len);
        self.send(command.as_bytes(), &mut response);


        // to ensure it's an even number of bytes, abscond with 1 byte from the payload.
        let prefix = [b'S', b'0', b'\r', write_info.data[0]];
        let remainder = &write_info.data[1..len];

        self.await_data_ready();
        {
            let _cs = self.cs.select();

            for chunk in prefix.chunks(2) {
                let mut xfer: [u8; 2] = [0; 2];
                xfer[1] = chunk[0];
                xfer[0] = chunk[1];
                //if chunk.len() == 2 {
                //} else {
                //xfer[0] = 0x0A
                //}

                //log::info!("transfer {:?}", xfer);
                self.spi.transfer(&mut xfer);
            }

            for chunk in remainder.chunks(2) {
                let mut xfer: [u8; 2] = [0; 2];
                xfer[1] = chunk[0];
                if chunk.len() == 2 {
                    xfer[0] = chunk[1]
                } else {
                    xfer[0] = 0x0A
                }

                //log::info!("transfer {:?}", xfer);
                self.spi.transfer(&mut xfer);
            }
        }
        self.await_data_ready();
        self.receive(&mut response);

        log::info!("response {}", core::str::from_utf8(&response).unwrap());

        Ok(len)
    }

    pub(crate) fn read(&mut self, socket_num: usize, buffer: &mut [u8]) -> Result<usize, ()> {
        self.process_backlog();

        let mut response = [0u8; 1100];

        let mut command: String<U8> = String::from("P0=");
        write!(command, "{}\r", socket_num);
        self.send(command.as_bytes(), &mut response);

        let mut len = buffer.len();
        if len > 1460 {
            len = 1460;
        }

        let mut command: String<U16> = String::from("R1=");
        write!(command, "{}\r", len);
        self.send(command.as_bytes(), &mut response);

        self.send("R2=0\r".as_bytes(), &mut response);
        self.send("R3=1\r".as_bytes(), &mut response);
        self.send("R?\r".as_bytes(), &mut response);

        self.await_data_ready();
        {
            let _cs = self.cs.select();

            let mut xfer = [b'0', b'R'];
            self.spi.transfer(&mut xfer);

            xfer = [b'\n', b'\r'];
            self.spi.transfer(&mut xfer);
        }

        self.await_data_ready();

        if let Ok(len) = self.receive(&mut response) {
            if let Ok((remainder, result)) = parser::read_data(&response) {
                if let parser::ReadResponse::Ok(result) = result {
                    for (i, b) in result.iter().enumerate() {
                        buffer[i] = *b;
                        //log::info!("b={}", *b as char);
                    }
                    return Ok(result.len());
                }
            }
        }
        //result
        Err(())
    }
}

