use heapless::{
    String,
    Vec,
    spsc::{Producer, Consumer},
    consts::*,
};
use crate::protocol::{Response, ConnectionType, ConnectInfo, WriteInfo};
use crate::socket::{Socket, State};
//use crate::network::EsWifiNetworkDriver;
use drogue_network::addr::HostSocketAddr;
use crate::arbiter::Arbiter;
use core::cell::RefCell;
use drogue_embedded_timer::Delay;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use crate::parser::join;

pub enum AdapterError {
    ReadError,
    NoAvailableSockets,
    SocketNotOpen,
}


#[derive(Debug)]
pub enum JoinError {
    Unknown,
    InvalidSsid,
    InvalidPassword,
    UnableToAssociate,
}

#[derive(Debug)]
pub enum JoinInfo<'a> {
    Open,
    Wep {
        ssid: &'a str,
        password: &'a str,
    },
}

impl JoinInfo<'_> {
    pub(crate) fn validate(&self) -> Result<&Self, JoinError> {
        match self {
            JoinInfo::Open => {
                Ok(self)
            }
            JoinInfo::Wep { ssid, password } => {
                if ssid.len() > 32 {
                    Err(JoinError::InvalidSsid)
                } else if password.len() > 32 {
                    Err(JoinError::InvalidPassword)
                } else {
                    Ok(self)
                }
            }
            _ => {
                Ok(self)
            }
        }
    }
}

pub struct Adapter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    pub(crate) arbiter: RefCell<Arbiter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>>,
    pub(crate) sockets: RefCell<[Socket; 4]>,
}

impl<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock> Adapter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    pub fn new(
        spi: Spi,
        cs: ChipSelectPin,
        ready: ReadyPin,
        wakeup: WakeupPin,
        reset: ResetPin,
        delay: Delay<'clock, Clock>,
    ) -> Result<Self, ()> {
        let mut arbiter = Arbiter::new(
            spi,
            cs,
            ready,
            wakeup,
            reset,
            delay,
        );

        Ok(Self {
            arbiter: RefCell::new(arbiter),
            sockets: RefCell::new(Socket::create()),
        })
    }


    pub fn join(&mut self, join_info: &JoinInfo) -> Result<(), JoinError> {
        join_info.validate()?;
        let mut arbiter = self.arbiter.borrow_mut();
        arbiter.join(join_info)
    }

    pub fn join_open(&mut self) -> Result<(), JoinError> {
        self.join(&JoinInfo::Open)
    }

    pub fn join_wep(&mut self, ssid: &str, password: &str) -> Result<(), JoinError> {
        self.join(
            &JoinInfo::Wep {
                ssid,
                password,
            }
        )
    }

    /*
    // ------------------------------------------------------------------------
    // Network-related
    // ------------------------------------------------------------------------

    pub fn open(&mut self) -> Result<usize, AdapterError> {
        if let Some((index, socket)) = self
            .sockets
            .iter_mut()
            .enumerate()
            .find(|(_, e)| e.is_closed())
        {
            socket.state = State::Open;
            return Ok(index);
        }

        Err(AdapterError::NoAvailableSockets)
    }

    pub fn connect_tcp(&mut self, socket_num: usize, remote: HostSocketAddr) -> Result<(), AdapterError> {
        let socket = &self.sockets[socket_num];
        if !socket.is_open() {
            return Err(AdapterError::SocketNotOpen);
        }

        self.requests.enqueue(
            Request::Connect(
                ConnectInfo {
                    socket_num,
                    connection_type: ConnectionType::Tcp,
                    remote,
                }
            )
        );

        let response = self.await_response();

        Ok(())
    }

    pub fn write(&mut self, socket_num: usize, data: &[u8]) -> nb::Result<usize, AdapterError> {
        let socket = &self.sockets[socket_num];
        if !socket.is_open() {
            return Err(nb::Error::from(AdapterError::SocketNotOpen));
        }

        let mut len = data.len();
        if len > 1024 {
            len = 1024;
        }

        self.requests.enqueue(
            Request::Write(
                WriteInfo {
                    socket_num,
                    data: Vec::from_slice(&data[0..len]).unwrap(),
                }
            )
        );

        let response = self.await_response();
        Ok(len)
    }

     */
}