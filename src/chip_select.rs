use embedded_hal::digital::v2::OutputPin;
use drogue_embedded_timer::Delay;
use embedded_time::duration::Milliseconds;

pub(crate) struct ChipSelect<'clock, Pin, Clock>
    where Pin: OutputPin,
          Clock: embedded_time::Clock,
{
    pin: Pin,
    delay: Delay<'clock, Clock>,
}

impl<'clock, Pin, Clock> ChipSelect<'clock, Pin, Clock>
    where Pin: OutputPin,
          Clock: embedded_time::Clock,
{
    /// Construct a new CS pin controller and set it high (unselected)
    pub(crate) fn new(mut pin: Pin, delay: Delay<'clock, Clock>) -> Self {
        pin.set_high();
        Self {
            pin,
            delay,
        }
    }

    pub(crate) fn select<'pin>(&'pin mut self) -> Selected<'pin, 'clock, Pin, Clock> {
        Selected::new(self)
    }

    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn set_high(&mut self) {
        self.pin.set_high();
    }
}

pub(crate) struct Selected<'pin, 'clock, Pin, Clock>
    where Pin: OutputPin,
          Clock: embedded_time::Clock,
{
    cs: &'pin mut ChipSelect<'clock, Pin, Clock>,
}

impl<'pin, 'clock, Pin, Clock> Selected<'pin, 'clock, Pin, Clock>
    where Pin: OutputPin,
          Clock: embedded_time::Clock,
{
    fn new(cs: &'pin mut ChipSelect<'clock, Pin, Clock>) -> Self {
        cs.set_low();
        cs.delay.delay( Milliseconds(10u32));
        Self {
            cs
        }
    }
}

impl<Pin, Clock> Drop for Selected<'_, '_, Pin, Clock>
    where Pin: OutputPin,
          Clock: embedded_time::Clock,
{
    fn drop(&mut self) {
        self.cs.set_high();
        self.cs.delay.delay( Milliseconds(10u32));
    }
}
