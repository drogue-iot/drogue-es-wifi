

use crate::protocol::Response;
use nom::{
    do_parse,
    named,
    alt,
    tag,
    char,
    take_until,
};
use heapless::String;

named!(
    pub ok,
    tag!("OK\r\n")
);

named!(
    pub prompt,
    tag!("> ")
);

// [JOIN   ] drogue,192.168.1.174,0,0
#[rustfmt::skip]
named!(
    pub join<Response>,
    do_parse!(
        tag!("\r\n") >>
        tag!("[JOIN   ] ") >>
        ssid: take_until!(",") >>
        char!(',') >>
        ip: take_until!(",") >>
        char!(',') >>
        tag!("0,0") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        ( {
            Response::Joined(String::from(core::str::from_utf8(ssid).unwrap()))
        } )
    )
);

named!(
    pub join_response<Response>,
    alt!(
        join
    )
);
