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
use crate::protocol::{Response};
