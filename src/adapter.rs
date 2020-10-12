use heapless::{
    String,
    Vec,
    spsc::{Producer, Consumer},
    consts::*,
};
use crate::socket::{Socket, State};
//use crate::network::EsWifiNetworkDriver;
use drogue_network::addr::HostSocketAddr;
use crate::arbiter::{Arbiter, SpiError};
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

#[derive(Debug)]
pub enum ConnectError {
    SpiError(SpiError),
    ConnectionFailed,
}

#[derive(Debug)]
pub enum CloseError {
    SpiError(SpiError),
    Error,
}

#[derive(Debug)]
pub enum WriteError {
    Error,
    SpiError(SpiError)
}

#[derive(Debug)]
pub enum ReadError {
    Error,
    SpiError(SpiError),
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


/// eS-WiFi Adapter, over SPI
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
    pub(crate) clock: &'clock Clock,
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
    /// Create a new eS-WiFi Adapter.
    ///
    /// * `spi`: The SPI transfer interface (u8).
    /// * `cs`: The chip-select pin for the adapter.
    /// * `ready`: The input pin to know when the adapter is ready for a data phase.
    /// * `wakeup`: The adapter's wake-up pin.
    /// * `reset`: The adapter's reset pin.
    /// * `clock`: A clock capable of 10ms precision
    pub fn new(
        spi: Spi,
        cs: ChipSelectPin,
        ready: ReadyPin,
        wakeup: WakeupPin,
        reset: ResetPin,
        clock: &'clock Clock,
    ) -> Result<Self, ()> {
        let mut arbiter = Arbiter::new(
            spi,
            cs,
            ready,
            wakeup,
            reset,
            clock,
        );

        Ok(Self {
            arbiter: RefCell::new(arbiter),
            sockets: RefCell::new(Socket::create()),
            clock,
        })
    }

    /// Join a WiFi access point.
    pub fn join(&mut self, join_info: &JoinInfo) -> Result<(), JoinError> {
        join_info.validate()?;
        let mut arbiter = self.arbiter.borrow_mut();
        arbiter.join(join_info)
    }

    /// Join an open WiFi access point.
    pub fn join_open(&mut self) -> Result<(), JoinError> {
        self.join(&JoinInfo::Open)
    }

    /// Join a WEP-secured WiFI access point.
    pub fn join_wep(&mut self, ssid: &str, password: &str) -> Result<(), JoinError> {
        self.join(
            &JoinInfo::Wep {
                ssid,
                password,
            }
        )
    }

}