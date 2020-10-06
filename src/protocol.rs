
use heapless::{
    String,
    consts::*,
};

#[derive(Debug)]
pub enum JoinInfo {
    Open,
    Wep{ssid: String<U32>, password: String<U64>},
}

#[derive(Debug)]
pub enum Request {
    Join(JoinInfo),
}

#[derive(Debug)]
pub enum Response {
    Ok(),
    Error(),
    Joined(String<U32>),
}
