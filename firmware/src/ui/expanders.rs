use super::{OP1, OP2, UiErr};
use embedded_hal::blocking::i2c::{Read as i2c_Read, Write as i2c_Write};
use core::marker::PhantomData;

pub struct Expanders<T: i2c_Write + i2c_Read> {
    i2c_type: PhantomData<T>,
}

impl <T: i2c_Write + i2c_Read> Expanders<T> {
    pub fn new<>(i2c: &mut T)->Result<Self, UiErr> {
        // Init expander 3 (Op sel switches and LEDs, upper-left bank of 8 argument switches)
        i2c.write(0x23, &[0x06, 0xFF]).map_err(UiErr::new)?; // Turn off LEDs
        i2c.write(0x23, &[0x5C, 0b00000100]).map_err(UiErr::new)?; // Open-drain output
        i2c.write(0x23, &[0x0C, 0xFF, 0xFF, 0x00]).map_err(UiErr::new)?; // DDR
        i2c.write(0x23, &[0x08, 0x00, 0xEE, 0x00]).map_err(UiErr::new)?; // Invert inputs
        // Minimum drive strength
        i2c.write(0x23, &[0x40, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF]).map_err(UiErr::new)?;
        i2c.write(0x23, &[0x4C, 0xFF, 0xFF, 0x00]).map_err(UiErr::new)?; // Enable pull-ups

        // Init expander 2 (All argument switches except upper-left bank)
        i2c.write(0x22, &[0x0C, 0xFF, 0xFF, 0xFF]).map_err(UiErr::new)?; // DDR
        i2c.write(0x22, &[0x08, 0x00, 0x00, 0x00]).map_err(UiErr::new)?; // Invert inputs
        i2c.write(0x22, &[0x4C, 0xFF, 0xFF, 0xFF]).map_err(UiErr::new)?; // Enable pull-ups

        // Init expander 0
        i2c.write(0x20, &[0x06, 0xF0]).map_err(UiErr::new)?; // Turn off LEDs
        i2c.write(0x20, &[0x5C, 0b00000100]).map_err(UiErr::new)?; // Open-drain output
        i2c.write(0x20, &[0x0C, 0xFF, 0xFF, 0x0F]).map_err(UiErr::new)?; // DDR
        i2c.write(0x20, &[0x08, 0x00, 0x00, 0x00]).map_err(UiErr::new)?; // Invert inputs
        // Minimum drive strength
        i2c.write(0x20, &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]).map_err(UiErr::new)?;
        i2c.write(0x20, &[0x4C, 0xFF, 0xFF, 0x0F]).map_err(UiErr::new)?; // Enable pull-ups

        Ok(Expanders{i2c_type: PhantomData})
    }

    /// Read operator select sliders and EN switches
    pub fn read_opsel_en(&self, i2c: &mut T, op1: &mut Option<OP1>, op2: &mut Option<OP2>,
                         op1_en: &mut bool, op2_en: &mut bool)->Result<(), UiErr> {
        let mut buf: [u8; 1] = [0];
        i2c.write(0x23, &[0x01]).map_err(UiErr::new)?;
        i2c.read(0x23, &mut buf).map_err(UiErr::new)?;

        *op1 = match buf[0] & 0b11100000 {
            0b00000000 => Some(OP1::RND),
            0b00100000 => Some(OP1::OSC),
            0b01000000 => Some(OP1::MUL),
            0b10000000 => Some(OP1::AND),
            _          => None,
        };
        *op2 = match buf[0] & 0b00001110 {
            0b00000000 => Some(OP2::SUB),
            0b00000010 => Some(OP2::MSK),
            0b00000100 => Some(OP2::XOR),
            0b00001000 => Some(OP2::OR),
            _          => None,
        };

        *op1_en = buf[0] & 0b00000001 != 0;
        *op2_en = buf[0] & 0b00010000 != 0;

        Ok(())
    }

    /// Read argument switches
    pub fn read_args(&self, i2c: &mut T, op1_args: &mut u16, op2_args: &mut u16)->Result<(), UiErr> {
        let mut buf1: [u8; 1] = [0; 1];
        i2c.write(0x23, &[0x00]).map_err(UiErr::new)?;
        i2c.read(0x23, &mut buf1).map_err(UiErr::new)?;
        let mut buf2: [u8; 3] = [0; 3];
        i2c.write(0x22, &[0x00]).map_err(UiErr::new)?;
        i2c.read(0x22, &mut buf2).map_err(UiErr::new)?;

        *op1_args = u16::from_le_bytes([buf2[0], buf1[0]]);
        *op2_args = u16::from_le_bytes([buf2[1], buf2[2]]);

        Ok(())
    }

    /// Read front-panel auxiliary buttons
    pub fn read_buttons(&self, i2c: &mut T, ret: &mut u8)->Result<(), UiErr> {
        let mut buf: [u8; 1] = [0];
        i2c.write(0x20, &[0x01]).map_err(UiErr::new)?;
        i2c.read(0x20, &mut buf).map_err(UiErr::new)?;
        *ret = buf[0];
        Ok(())
    }

    /// Read the four settings switches
    pub fn read_settings(&self, i2c: &mut T,sw0: &mut bool,sw1: &mut bool,
                         sw2: &mut bool, sw3: &mut bool)->Result<(), UiErr> {
        let mut buf: [u8; 1] = [0];
        i2c.write(0x20, &[0x02]).map_err(UiErr::new)?;
        i2c.read(0x20, &mut buf).map_err(UiErr::new)?;
        *sw0 = buf[0] & 0b00000001 != 0;
        *sw1 = buf[0] & 0b00000010 != 0;
        *sw2 = buf[0] & 0b00000100 != 0;
        *sw3 = buf[0] & 0b00001000 != 0;

        Ok(())
    }

    pub fn write_opsel_leds(&self, i2c: &mut T, op1: &OP1, op2: &OP2,
                            op1_en: &bool, op2_en: &bool)->Result<(), UiErr> {
        let op1_leds = if *op1_en == true {
            match *op1 {
                OP1::RND => 0b11101111,
                OP1::OSC => 0b11011111,
                OP1::MUL => 0b10111111,
                OP1::AND => 0b01111111,
            }
        } else { 0xFF };

        let op2_leds = if *op2_en == true {
            match *op2 {
                OP2::SUB => 0b11111110,
                OP2::MSK => 0b11111101,
                OP2::XOR => 0b11111011,
                OP2::OR  => 0b11110111,
            }
        } else { 0xFF };

        let leds = op1_leds & op2_leds;
        i2c.write(0x23, &[0x06, leds]).map_err(UiErr::new)?;
        Ok(())
    }
}
