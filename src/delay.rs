//use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::prelude::*;
use embedded_hal::timer::CountDown;

use nb::block;
use embedded_hal::blocking::delay::DelayMs;
use embedded_time::duration::{Duration, Milliseconds};

pub(crate) struct DelayTimer<CD>
    where
        CD: CountDown,
        CD::Time: Duration + From<Milliseconds>,
{
    count_down: CD,
}

impl<CD> DelayTimer<CD>
    where
        CD: CountDown,
        CD::Time: Duration + From<Milliseconds>,
{
    pub(crate) fn new(count_down: CD) -> Self {
        Self {
            count_down,
        }
    }
}

impl<CD> DelayMs<u32> for DelayTimer<CD>
    where
        CD: CountDown,
        CD::Time: Duration + From<Milliseconds>,
{
    fn delay_ms(&mut self, ms: u32) {
        let duration = Milliseconds(ms);
        self.count_down.start( duration);
        block!(self.count_down.wait());
    }
}
