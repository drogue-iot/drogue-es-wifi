use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayMs;

pub(crate) struct Selected<'a, CS, Delay>
    where CS: OutputPin + 'a,
          Delay: DelayMs<u32>,
{
    cs: &'a mut CS,
    delay: &'a mut Delay,
}

pub(crate) trait Selectable<'a, CS, Delay>
    where CS: OutputPin + 'a,
          Delay: DelayMs<u32>,
{
    fn select(&'a mut self, delay: &'a mut Delay) -> Selected<'a, CS, Delay>;
}

impl<'a, CS, Delay> Selectable<'a, CS, Delay> for CS
    where CS: OutputPin + 'a,
          Delay: DelayMs<u32>,
{
    fn select(&'a mut self, delay: &'a mut Delay) -> Selected<'a, CS, Delay> {
        self.set_low().unwrap_or(());
        delay.delay_ms(10);
        log::info!("select CS");
        Selected {
            cs: self,
            delay,
        }
    }

}

impl<'a, CS, Delay> Drop for Selected<'_, CS, Delay>
    where CS: OutputPin,
          Delay: DelayMs<u32>,
{
    fn drop(&mut self) {
        log::info!("de-select CS");
        self.cs.set_high().unwrap_or(());
        self.delay.delay_ms(10);
    }
}