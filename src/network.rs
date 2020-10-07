use drogue_network::tcp::{TcpStack, Mode, TcpError};
use drogue_network::addr::HostSocketAddr;

use nb;
use core::cell::RefCell;
use crate::adapter::{Adapter, AdapterError};
use nb::Error;

pub struct TcpSocket(usize);

#[derive(Debug)]
pub enum EsWifiTcpError {
    NoAvailableSockets,
    SocketNotOpen,
}

impl Into<TcpError> for EsWifiTcpError {
    fn into(self) -> TcpError {
        match self {
            EsWifiTcpError::NoAvailableSockets => {
                TcpError::NoAvailableSockets
            }
            EsWifiTcpError::SocketNotOpen => {
                TcpError::SocketNotOpen
            }
        }
    }
}

impl Into<EsWifiTcpError> for AdapterError {
    fn into(self) -> EsWifiTcpError {
        match self {
            AdapterError::NoAvailableSockets => {
                EsWifiTcpError::NoAvailableSockets
            }
            AdapterError::SocketNotOpen => {
                EsWifiTcpError::SocketNotOpen
            }
        }
    }
}


pub struct EsWifiNetworkDriver<'q> {
    adapter: RefCell<Adapter<'q>>,
}

impl<'q> EsWifiNetworkDriver<'q> {
    pub(crate) fn new(adapter: Adapter<'q>) -> Self {
        Self {
            adapter: RefCell::new(adapter),
        }
    }
}

impl<'q> TcpStack for EsWifiNetworkDriver<'q> {
    type TcpSocket = TcpSocket;
    type Error = EsWifiTcpError;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        match adapter.open() {
            Ok(socket_num) => {
                Ok(TcpSocket(socket_num))
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    fn connect(&self, socket: Self::TcpSocket, remote: HostSocketAddr) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        match adapter.connect_tcp(socket.0, remote) {
            Ok(socket_num) => {
                Ok(socket)
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        match adapter.write(socket.0, buffer) {
            Ok(len) => {
                Ok(len)
            }
            Err(e) => {
                match e {
                    Error::Other(e) => {
                        let e: EsWifiTcpError = e.into();
                        Err(Error::from(e))
                    }
                    Error::WouldBlock => {
                        Err(Error::WouldBlock)
                    }
                }
            }
        }
    }

    fn read(&self, socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        unimplemented!()
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        unimplemented!()
    }
}