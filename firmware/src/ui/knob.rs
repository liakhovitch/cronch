use embedded_hal::adc::Channel;
use embedded_hal::prelude::{
    _embedded_hal_adc_OneShot,
};
use rp2040_hal::adc::Adc;
use super::UiErr;

pub struct Knob<T: Channel<Adc, ID = u8>> {
    pin: T,
    direction: bool,
    prev: u16,
}

impl <T: Channel<Adc, ID = u8>> Knob<T> {
    pub fn new(pin: T) -> Self {
        Knob{
            pin,
            direction: true,
            prev: 0,
        }
    }

    pub fn read(&mut self, adc: &mut Adc) -> Result<Option<u16>, UiErr> {
        let mut reading_avg: u32 = 0;
        for _ in 0..128 {
            let reading: u16 = adc.read(&mut self.pin).map_err(UiErr::new)?;
            reading_avg += reading as u32;
        }
        reading_avg >>= 7;
        let val = (( 1<<12 ) - 1) - reading_avg as u16;

        // Awful hysteresis logic
        let threshold = 128;
        match (self.direction, val > self.prev) {
            (true, true) => {
                self.prev = val + (threshold / 2);
                Ok(Some(val))
            },
            (true, false) => {
                if val <= self.prev - threshold {
                    self.direction = false;
                    self.prev = val + (threshold / 2);
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            },
            (false, false) => {
                self.prev = if val > (threshold / 2) { val - (threshold/2) }
                else { 0 };
                Ok(Some(val))
            },
            (false, true) => {
                if val > self.prev + threshold {
                    self.direction = true;
                    self.prev = if val > (threshold / 2) { val - (threshold/2) }
                    else { 0 };
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            },
        }
    }
}
