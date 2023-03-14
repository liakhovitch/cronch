use super::{UiErr, UiInput, UiOutput};
use embedded_hal::blocking::i2c::{Read as i2c_Read, Write as i2c_Write};
use rp2040_hal::sio::HwDivider;
use rp2040_hal::timer::{Instant, Timer};
use core::marker::PhantomData;

enum StripState {
    Normal,
    Fdbk(Instant),
    Clk(Instant),
    Mix(Instant),
    Init(Instant),
}

// We can't own the actual I2C resource or a mutable reference, as the bus is shared.
// Thus, a reference to an I2C resource must be passed into each call to a member function.
// The associated type and PhantomData ensure that we're at least using the same I2C peripheral
//     across different member calls on the same LedStrip object.
pub struct LedStrip<'a, 'b, T: i2c_Write + i2c_Read> {
    i2c_type: PhantomData<T>,
    state: StripState,
    init_time: u32,
    knob_time: u32,
    timer: &'a Timer,
    div: &'b HwDivider,
}

impl <'a, 'b, T: i2c_Write + i2c_Read> LedStrip<'a, 'b, T> {

    pub fn new<>(i2c: &mut T, timer: &'a Timer, div: &'b HwDivider,
                 init_time: u16, knob_time: u16) -> Result<Self, UiErr> {
        i2c.write(0x60, &[
            0x00, 0x00,
            0x0D, 0b00001000,
            0x0F, 0x00,
            0x0C, 0x00,
        ]).map_err(UiErr::new)?;
        let state = StripState::Init(timer.get_counter());
        Ok(LedStrip{
            i2c_type: PhantomData,
            state,
            init_time: init_time as u32,
            knob_time: knob_time as u32,
            timer,
            div,
        })
    }

    fn write(&self, i2c: &mut T, red: u32, green: u32)->Result<(), UiErr>{
        i2c.write(0x60, &[
            1, green.to_le_bytes()[0],
            2, green.to_le_bytes()[1],
            3, green.to_le_bytes()[2],
            4, green.to_le_bytes()[3],
            5, red.to_le_bytes()[0],
            6, red.to_le_bytes()[1],
            7, red.to_le_bytes()[2],
            8, red.to_le_bytes()[3],
            0x0c, 0x00
        ]).map_err(UiErr::new)?;
        Ok(())
    }

    pub fn display_fdbk(&mut self){
        self.state = StripState::Fdbk(self.timer.get_counter());
    }

    pub fn display_clk(&mut self){
        self.state = StripState::Clk(self.timer.get_counter());
    }

    pub fn display_mix(&mut self){
        self.state = StripState::Mix(self.timer.get_counter());
    }

    pub fn animate_init(&mut self, start: Instant) -> (u32, u32){
        let delta = (self.timer.get_counter() - start).to_millis() as u32;
        let time_passed = if delta < self.init_time {
            delta
        }
        else {
            self.state = StripState::Normal;
            self.init_time
        };
        let mut step = self.div.unsigned(time_passed * 160, self.init_time).quotient;
        if step != 0 {step -= 1}
        let disp = if step < 32{
            0xFFFFFFFF << (31 - step)
        }
        else if step < 64{
            0xFFFFFFFF >> (step - 32)
        }
        else if step < 96 {
            0x00
        }
        else if step < 128 {
            0xFFFFFFFF >> (127 - step)
        }
        else {
            0xFFFFFFFF << (step - 128)
        };
        (disp, disp)
    }

    pub fn animate_level(&mut self, start: Instant, knob: u32) -> (u32, u32) {
        let delta = (self.timer.get_counter() - start).to_millis() as u32;
        if delta > self.knob_time {
            self.state = StripState::Normal;
        }
        let disp: u32 = if knob < 32 {
            0xFFFFFFFF << (31 - knob)
        }
        else {
            0xFFFFFFFF
        };

        (disp, disp)
    }

    pub fn animate_balance(&mut self, start: Instant, knob: u32) -> (u32, u32) {
        let delta = (self.timer.get_counter() - start).to_millis() as u32;
        if delta > self.knob_time {
            self.state = StripState::Normal;
        }
        let red: u32 = if knob < 32 {
            0xFFFFFFFF << (31 - knob)
        }
        else {
            0xFFFFFFFF
        };
        let green: u32 = if knob < 32 {
            0xFFFFFFFF >> knob
        }
        else {
            0x00000000
        };

        (red, green)
    }

    pub fn update(&mut self, i2c: &mut T, out: &UiOutput, input: &UiInput) -> Result<(), UiErr> {
        let (red, green) = match self.state {
            StripState::Normal => {
                let red: u32 = 1u32.rotate_right(1 + (input.write_addr >> 11) as u32);
                let green: u32 = 1u32.rotate_right(1 + (input.read_addr >> 11) as u32);
                (red, green)
            }
            StripState::Fdbk(n) => {
                self.animate_level(n, (out.fdbk_knob >> 7) as u32)
            }
            StripState::Clk(n) => {
                self.animate_level(n, (out.clk_knob >> 7) as u32)
            }
            StripState::Mix(n) => {
                self.animate_balance(n, (out.mix_knob >> 7) as u32)
            }
            StripState::Init(n) => {
                self.animate_init(n)
            }
        };
        self.write(i2c, red, green)?;
        Ok(())
    }

}
