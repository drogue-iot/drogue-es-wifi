use embedded_hal::digital::v2::InputPin;

pub(crate) struct Ready<Pin>
    where Pin: InputPin
{
    pin: Pin
}

impl<Pin> Ready<Pin>
    where Pin: InputPin
{
    pub(crate) fn new(pin: Pin) -> Self {
        Self {
            pin
        }
    }

    pub(crate) fn is_ready(&self) -> bool {
        self.pin.is_high().unwrap_or(false)
    }

}