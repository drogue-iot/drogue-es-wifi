use heapless::{
    String,
    Vec,
    spsc::{Producer, Consumer},
    consts::*,
};
use crate::protocol::{Request, Response, JoinInfo, ConnectionType, ConnectInfo, WriteInfo};
use crate::socket::{Socket, State};
use crate::network::EsWifiNetworkDriver;
use drogue_network::addr::HostSocketAddr;

pub enum AdapterError {
    NoAvailableSockets,
    SocketNotOpen,
}

pub struct Adapter<'q> {
    requests: Producer<'q, Request, U1>,
    responses: Consumer<'q, Response, U1>,
    sockets: [Socket; 4],
}

impl<'q> Adapter<'q> {
    pub fn new(
        producer: Producer<'q, Request, U1>,
        consumer: Consumer<'q, Response, U1>,
    ) -> Self {
        Self {
            requests: producer,
            responses: consumer,
            sockets: Socket::create(),
        }
    }

    fn await_response(&mut self) -> Response {
        loop {
            if let Some(response) = self.responses.dequeue() {
                log::info!("response {:?}", response);
                return response;
            }
        }
    }

    pub fn join(&mut self, ssid: &str, password: &str) -> Result<Response, ()> {
        self.requests.enqueue(
            Request::Join(JoinInfo::Wep {
                ssid: String::from(ssid),
                password: String::from(password),
            })
        );

        Ok(self.await_response())
        //loop {
        //if let Some(response) = self.responses.dequeue() {
        //log::info!("response {:?}", response);
        //return Ok(response);
        //}
        //}
    }

    // ------------------------------------------------------------------------
    // Network-related
    // ------------------------------------------------------------------------

    pub fn into_network_driver(self) -> EsWifiNetworkDriver<'q> {
        EsWifiNetworkDriver::new(self)
    }

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
        if ! socket.is_open() {
            return Err( AdapterError::SocketNotOpen )
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
        if ! socket.is_open() {
            return Err(nb::Error::from( AdapterError::SocketNotOpen ));
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
}