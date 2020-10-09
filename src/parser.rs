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
use drogue_nom_utils::{
    parse_usize,
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

pub(crate) enum ConnectResponse {
    Ok,
    Error,
}


named!(
    pub(crate) connected<ConnectResponse>,
    do_parse!(
        tag!("\r\n") >>
        tag!("[TCP  RC] Connecting to ") >>
        take_until!( "\r\n") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            ConnectResponse::Ok
        )
    )
);

named!(
    pub(crate) connection_failure<ConnectResponse>,
    do_parse!(
        take_until!( "ERROR" ) >>
        error >>
        (
            ConnectResponse::Error
        )
    )
);

named!(
    pub(crate) connect_response<ConnectResponse>,
    alt!(
        complete!(connected)
        | complete!(connection_failure)
    )
);

#[derive(Debug)]
pub(crate) enum CloseResponse {
    Ok,
    Error,
}

named!(
    pub(crate) closed<CloseResponse>,
    do_parse!(
        tag!("\r\n") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            CloseResponse::Ok
        )
    )
);

named!(
    pub(crate) close_error<CloseResponse>,
    do_parse!(
        tag!("\r\n") >>
        take_until!( "ERROR" ) >>
        error >>
        prompt >>
        (
            CloseResponse::Error
        )
    )
);

named!(
    pub(crate) close_response<CloseResponse>,
    alt!(
          complete!(closed)
        | complete!(close_error)
    )
);

#[derive(Debug)]
pub(crate) enum WriteResponse {
    Ok(usize),
    Error,
}

named!(
    pub(crate) write_ok<WriteResponse>,
    do_parse!(
        tag!("\r\n") >>
        len: parse_usize >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            WriteResponse::Ok(len)
        )
    )
);

named!(
    pub(crate) write_error<WriteResponse>,
    do_parse!(
        tag!("\r\n") >>
        tag!("-1") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            WriteResponse::Error
        )
    )
);

named!(
    pub(crate) write_response<WriteResponse>,
    alt!(
          complete!(write_ok)
        | complete!(write_error)
    )
);


#[derive(Debug)]
pub(crate) enum ReadResponse<'a> {
    Ok(&'a [u8]),
    Err,
}

named!(
    pub(crate) read_data<ReadResponse>,
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

named!(
    pub(crate) read_error<ReadResponse>,
    do_parse!(
        tag!("\r\n") >>
        tag!("-1") >>
        tag!("\r\n") >>
        ok >>
        prompt >>
        (
            ReadResponse::Err
        )
    )
);

named!(
    pub(crate) read_response<ReadResponse>,
    alt!(
          complete!(read_data)
        | complete!(read_error)
    )
);

