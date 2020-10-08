use drogue_network::tcp::{TcpStack, Mode, TcpError};
use drogue_network::addr::HostSocketAddr;

use nb;
use core::cell::RefCell;
use crate::adapter::{Adapter, AdapterError};
use nb::Error;
use crate::socket::State;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use crate::protocol::{ConnectInfo, ConnectionType, WriteInfo};

pub struct TcpSocket(usize);

impl<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock> TcpStack for Adapter<'clock, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    type TcpSocket = TcpSocket;
    type Error = TcpError;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        if let Some((index, socket)) = self
            .sockets
            .borrow_mut()
            .iter_mut()
            .enumerate()
            .find(|(_, e)| e.is_closed())
        {
            socket.state = State::Open;
            return Ok(TcpSocket(index));
        }

        Err(TcpError::NoAvailableSockets)
    }

    fn connect(&self, tcp_socket: Self::TcpSocket, remote: HostSocketAddr) -> Result<Self::TcpSocket, Self::Error> {
        let socket = &self.sockets.borrow()[tcp_socket.0];
        if !socket.is_open() {
            return Err(TcpError::SocketNotOpen);
        }

        let mut arbiter = self.arbiter.borrow_mut();

        let response = arbiter.connect(
            &ConnectInfo {
                socket_num: tcp_socket.0,
                connection_type: ConnectionType::Tcp,
                remote,
            }
        );

        if response.is_ok() {
            return Ok(tcp_socket)
        }

        Err(TcpError::ConnectionRefused)
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn write(&self, tcp_socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = &self.sockets.borrow()[tcp_socket.0];
        if !socket.is_open() {
            return Err(nb::Error::from(TcpError::SocketNotOpen));
        }

        let mut arbiter = self.arbiter.borrow_mut();

        let result = arbiter.write(
            &WriteInfo {
                socket_num: tcp_socket.0,
                data: buffer,
            }
        );

        if result.is_ok() {
            Ok(result.unwrap())
        } else {
            Err(nb::Error::from(TcpError::WriteError))
        }
    }

    fn read(&self, tcp_socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let socket = &self.sockets.borrow()[tcp_socket.0];
        if ! socket.is_open() {
            return Err(nb::Error::from(TcpError::SocketNotOpen));
        }

        let mut arbiter = self.arbiter.borrow_mut();

        let result = arbiter.read(
            tcp_socket.0,
            buffer,
        );

        if result.is_ok() {
            return Ok(result.unwrap())
        }

        Err(nb::Error::Other(TcpError::ReadError))
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
