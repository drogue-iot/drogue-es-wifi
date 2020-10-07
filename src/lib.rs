#![no_std]

pub mod adapter;
pub mod arbiter;
mod parser;
pub mod protocol;
mod chip_select;
mod ready;
mod socket;
pub mod network;

use drogue_embedded_timer::Delay;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, InputPin};

use crate::arbiter::Arbiter;
use crate::adapter::Adapter;
use heapless::{
    consts::*,
    spsc::Queue
};
use crate::protocol::{Request, Response};

pub fn initialize<'clock, 'q, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>(
    spi: Spi,
    cs: ChipSelectPin,
    ready: ReadyPin,
    wakeup: WakeupPin,
    reset: ResetPin,
    delay: Delay<'clock, Clock>,
    request_queue: &'q mut Queue<Request, U1>,
    response_queue: &'q mut Queue<Response, U1>,
) -> (Adapter<'q>, Arbiter<'clock, 'q, Spi, ChipSelectPin, ReadyPin, WakeupPin, ResetPin, Clock>)
    where
        Spi: Transfer<u8>,
        ChipSelectPin: OutputPin,
        ReadyPin: InputPin,
        WakeupPin: OutputPin,
        ResetPin: OutputPin,
        Clock: embedded_time::Clock + 'clock
{
    let (request_producer, request_consumer) = request_queue.split();
    let (response_producer, response_consumer) = response_queue.split();
    (
        Adapter::new(
            request_producer,
            response_consumer,
        ),
        Arbiter::new(
            spi,
            cs,
            ready,
            wakeup,
            reset,
            delay,
            request_consumer,
            response_producer,
        ),
    )
}