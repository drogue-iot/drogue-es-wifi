use heapless::{String, consts::*, Vec};
use drogue_network::tcp::Mode;
use drogue_network::addr::HostSocketAddr;



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
pub struct WriteInfo<'a> {
    pub socket_num: usize,
    pub data: &'a [u8],
}

/*
#[derive(Debug)]
pub enum Request {
    Join(JoinInfo),
    Connect(ConnectInfo),
    Write(WriteInfo),
}
 */

#[derive(Debug)]
pub enum Response {
    Ok(),
    Error(),
    Joined(String<U32>),
}


