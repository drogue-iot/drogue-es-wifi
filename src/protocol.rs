use heapless::{String, consts::*, Vec};
use drogue_network::tcp::Mode;
use drogue_network::addr::HostSocketAddr;

#[derive(Debug)]
pub enum JoinInfo {
    Open,
    Wep { ssid: String<U32>, password: String<U64> },
}

#[derive(Debug)]
pub enum ConnectionType {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub struct ConnectInfo {
    pub socket_num: usize,
    pub connection_type: ConnectionType,
    pub remote: HostSocketAddr,
}

#[derive(Debug)]
pub struct WriteInfo {
    pub socket_num: usize,
    pub data: Vec<u8, U1024>,
}

#[derive(Debug)]
pub enum Request {
    Join(JoinInfo),
    Connect(ConnectInfo),
    Write(WriteInfo),
}

#[derive(Debug)]
pub enum Response {
    Ok(),
    Error(),
    Joined(String<U32>),
}
