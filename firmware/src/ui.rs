use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write as i2c_Write;
use embedded_hal::prelude::_embedded_hal_blocking_i2c_Read as i2c_Read;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct UiErr;

impl UiErr{
    fn new<T>(_: T)->Self{
        UiErr{}
    }
}

#[derive(Default)]
pub struct UiInput {
    pub read_addr: u16,
    pub write_addr: u16,
}

#[derive(Default)]
pub struct UiOutput {
    pub op1: OP1,
    pub op2: OP2,
    pub op1_en: bool,
    pub op2_en: bool,
    pub op1_arg: u16,
    pub op2_arg: u16,
    pub rev: bool,
    pub wr_prot: bool,
    pub fb_lr_swap: bool,
    pub fb_phase_flip: bool,
    pub fdbk_knob: u16,
    pub clk_knob: u16,
    pub mix_knob: u16,
}

#[derive(Default)]
pub enum OP1 {
    #[default]
    AND,
    MUL,
    OSC,
    RND,
}

#[derive(Default)]
pub enum OP2 {
    #[default]
    OR,
    XOR,
    MSK,
    SUB,
}

// We can't own the actual I2C resource or a mutable reference, as the bus is shared.
// Thus, a reference to an I2C resource must be passed into each call to a member function.
// The associated type and PhantomData ensure that we're at least using the same I2C peripheral
//     across different member calls on the same LedStrip object.
pub struct LedStrip<T: i2c_Write + i2c_Read> {
    i2c_type: PhantomData<T>,
}

impl <T: i2c_Write + i2c_Read> LedStrip<T> {
    pub fn new<>(i2c: &mut T)->Result<Self, UiErr> {
        i2c.write(0x60, &[
            0x00, 0x00,
            0x0D, 0b00001000,
            0x0F, 0x00,
            0x0C, 0x00,
        ]).map_err(UiErr::new)?;
        Ok(LedStrip{i2c_type: PhantomData})
    }

    pub fn write(&self, i2c: &mut T, red: u32, green: u32)->Result<(), UiErr>{
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
}

pub struct Expanders<T: i2c_Write + i2c_Read> {
    i2c_type: PhantomData<T>,
}

impl <T: i2c_Write + i2c_Read> Expanders<T> {
    pub fn new<>(i2c: &mut T)->Result<Self, UiErr> {
        // Init expander 3 (Op sel switches and LEDs, upper-left bank of 8 argument switches)
        i2c.write(0x23, &[0x06, 0xFF]).map_err(UiErr::new)?; // Turn off LEDs
        i2c.write(0x23, &[0x5C, 0b00000100]).map_err(UiErr::new)?; // Open-drain output
        i2c.write(0x23, &[0x0C, 0xFF, 0xFF, 0x00]).map_err(UiErr::new)?; // DDR
        i2c.write(0x23, &[0x08, 0xFF, 0xFF, 0x00]).map_err(UiErr::new)?; // Invert inputs
        // Minimum drive strength
        i2c.write(0x23, &[0x40, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF]).map_err(UiErr::new)?;
        i2c.write(0x23, &[0x4C, 0xFF, 0xFF, 0x00]).map_err(UiErr::new)?; // Enable pull-ups

        // Init expander 2 (All argument switches except upper-left bank)
        i2c.write(0x22, &[0x0C, 0xFF, 0xFF, 0xFF]).map_err(UiErr::new)?; // DDR
        i2c.write(0x22, &[0x08, 0xFF, 0xFF, 0xFF]).map_err(UiErr::new)?; // Invert inputs
        i2c.write(0x22, &[0x4C, 0xFF, 0xFF, 0xFF]).map_err(UiErr::new)?; // Enable pull-ups

        // Init expander 0
        i2c.write(0x20, &[0x06, 0xF0]).map_err(UiErr::new)?; // Turn off LEDs
        i2c.write(0x20, &[0x5C, 0b00000100]).map_err(UiErr::new)?; // Open-drain output
        i2c.write(0x20, &[0x0C, 0xFF, 0xFF, 0x0F]).map_err(UiErr::new)?; // DDR
        i2c.write(0x20, &[0x08, 0xFF, 0xFF, 0x0F]).map_err(UiErr::new)?; // Invert inputs
        // Minimum drive strength
        i2c.write(0x20, &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]).map_err(UiErr::new)?;
        i2c.write(0x20, &[0x4C, 0xFF, 0xFF, 0x0F]).map_err(UiErr::new)?; // Enable pull-ups

        Ok(Expanders{i2c_type: PhantomData})
    }

    /// Read operator select sliders and EN switches
    pub fn read_opsel_en(&self, i2c: &mut T, op1: &mut OP1, op2: &mut OP2,
                         op1_en: &mut bool, op2_en: &mut bool)->Result<(), UiErr> {
        let mut buf: [u8; 1] = [0];
        i2c.write(0x23, &[0x01]).map_err(UiErr::new)?;
        i2c.read(0x23, &mut buf).map_err(UiErr::new)?;

        *op1 = match buf[0] & 0b11100000 {
            0b00000000 => OP1::RND,
            0b00100000 => OP1::OSC,
            0b01000000 => OP1::MUL,
            _          => OP1::AND,
        };
        *op2 = match buf[0] & 0b00001110 {
            0b00000000 => OP2::SUB,
            0b00000010 => OP2::MSK,
            0b00000100 => OP2::XOR,
            _          => OP2::OR,
        };

        *op1_en = buf[0] & 0b00000001 == 0;
        *op2_en = buf[0] & 0b00010000 == 0;

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

        *op1_args = u16::from_le_bytes([buf1[0], buf2[0]]);
        *op2_args = u16::from_le_bytes([buf2[2], buf2[1]]);

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
