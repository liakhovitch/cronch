use embedded_hal::adc::Channel;
use embedded_hal::prelude::{
    _embedded_hal_adc_OneShot,
};
use rp2040_hal::adc::Adc;
use super::UiErr;

pub struct Knob<T: Channel<Adc, ID = u8>> {
    pin: T,
    prev: u16,
    shift: u32,
}

impl <T: Channel<Adc, ID = u8>> Knob<T> {
    pub fn new(pin: T, shift: u32) -> Self {
        Knob{
            pin,
            prev: 0,
            shift,
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
        if val < (self.prev << self.shift) || val >= ((self.prev + 2) << self.shift){
            self.prev = if val >= (1 << (self.shift - 1)) {
                (val - (1 << (self.shift - 1))) >> self.shift
            }
            else {
                0
            };
            Ok(Some(self.prev))
        }
        else {
            Ok(None)
        }
    }
}
