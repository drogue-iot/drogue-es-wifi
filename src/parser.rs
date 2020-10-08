use crate::protocol::{
    Response,
};
use nom::{
    do_parse,
    complete,
    named,
    named_args,
    alt,
    tag,
    char,
    take,
    take_until,
};
use heapless::String;
use crate::adapter::JoinError;

named!(
    pub ok,
    tag!("OK\r\n")
);

named!(
    pub error,
    tag!("ERROR\r\n")
);

named!(
    pub prompt,
    tag!("> ")
);

#[derive(Debug)]
pub(crate) enum JoinResponse {
    Ok,
    JoinError,
}

// [JOIN   ] drogue,192.168.1.174,0,0
#[rustfmt::skip]
named!(
    pub(crate) join<JoinResponse>,
    do_parse!(
        tag!("[JOIN   ] ") >>
        ssid: take_until!(",") >>
        char!(',') >>
        ip: take_until!(",") >>
        char!(',') >>
        tag!("0,0") >>
        tag!("\r\n") >>
        ok >>
        (
            JoinResponse::Ok
        )
        //( {
            //Response::Joined(String::from(core::str::from_utf8(ssid).unwrap()))
        //} )
    )
);

// [JOIN   ] drogue
// [JOIN   ] Failed
named!(
    pub(crate) join_error<JoinResponse>,
    do_parse!(
        take_until!( "ERROR" ) >>
        error >>
        (
            JoinResponse::JoinError
        )
    )
);

named!(
    pub(crate) join_response<JoinResponse>,
    do_parse!(
        tag!("\r\n") >>
        response:
        alt!(
              complete!(join)
            | complete!(join_error)
        ) >>
        prompt >>
        (
            response
        )

    )
);

named!(
    pub connect_response<Response>,
    alt!(
        connected
    )
);

named!(
    pub connected<Response>,
    do_parse!(
        tag!("\r\n") >>
        tag!("[TCP  RC] Connecting to ") >>
        take_until!( "\r\n") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            Response::Ok()
        )
    )
);

#[derive(Debug)]
pub enum ReadResponse<'a> {
    Ok(&'a [u8]),
    Err,
}

named!(
    pub read_data<ReadResponse>,
    do_parse!(
        tag!("\r\n") >>
        data: take_until!("\r\nOK\r\n>") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            ReadResponse::Ok(data)
        )
    )
);

