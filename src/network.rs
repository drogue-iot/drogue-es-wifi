use drogue_network::tcp::{TcpStack, Mode, TcpError, TcpImplError};
use drogue_network::addr::HostSocketAddr;

use nb;
use core::cell::RefCell;
use crate::adapter::{Adapter, AdapterError, ReadError};
use nb::Error;
use crate::socket::State;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use crate::arbiter::IpProtocol;
use embedded_time::duration::Milliseconds;

#[derive(Debug)]
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
            socket.mode = mode;
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
            IpProtocol::Tcp,
            tcp_socket.0,
            remote,
        );

        if response.is_ok() {
            return Ok(tcp_socket);
        }

        Err(TcpError::ConnectionRefused)
    }

    fn is_connected(&self, tcp_socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        let socket = &self.sockets.borrow()[tcp_socket.0];
        Ok(socket.is_connected())
    }

    fn write(&self, tcp_socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = &self.sockets.borrow()[tcp_socket.0];
        if !socket.is_open() {
            return Err(nb::Error::from(TcpError::SocketNotOpen));
        }

        let mut arbiter = self.arbiter.borrow_mut();

        let result = arbiter.write(tcp_socket.0, buffer);

        if result.is_ok() {
            Ok(result.unwrap())
        } else {
            Err(nb::Error::from(TcpError::WriteError))
        }
    }

    fn read(&self, tcp_socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let mut socket = &mut self.sockets.borrow_mut()[tcp_socket.0];
        if !socket.is_open() {
            return Err(nb::Error::from(TcpError::SocketNotOpen));
        }

        let mut arbiter = self.arbiter.borrow_mut();

        let mut timer = None;

        if let Mode::Timeout(ms) = socket.mode {
            timer = Some(
                self.clock.new_timer(Milliseconds(ms as u32)).start().unwrap()
            );
        }

        loop {
            let len = arbiter.read(tcp_socket.0, buffer).map_err(|_|
                {
                    socket.state = State::HalfClosed;
                    TcpError::ReadError
                })?;

            if len != 0 {
                return Ok(len);
            }

            if socket.is_non_blocking() {
                return Err(nb::Error::WouldBlock);
            }

            if let Some(ref timer) = timer {
                if let Ok(true) = timer.is_expired() {
                    return Ok(0)
                }
            }
        }
    }

    fn close(&self, tcp_socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let mut socket = &mut self.sockets.borrow_mut()[tcp_socket.0];
        if socket.is_open() {
            socket.state = State::Closed;
            let mut arbiter = self.arbiter.borrow_mut();
            arbiter.close(tcp_socket.0).map_err(|_| TcpError::Impl(TcpImplError::Unknown))?;
        } else {
            socket.state = State::Closed;
        }

        Ok(())
    }
}
