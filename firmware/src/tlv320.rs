use debugless_unwrap::DebuglessUnwrap;

const INIT_CMD_1:&[&[u8]] = &[
    // Clock setup
    &[00, 0x00], // Select page 0
    &[11, 0x81], // NDAC=1, divider enabled
    &[12, 0x82], // MDAC=2, divider enabled
    &[18, 0x81], // NADC=1, divider enabled
    &[19, 0x82], // MADC=2, divider enabled
    &[13, 0x00], // DAC OSR MSB
    &[14, 32],   // DAC OSR LSB: set to 32 for 192KHz sample rate
    &[60, 18],   // Select DAC processing block 18 (Filter C, Stereo, IIR, 4 biquads, DRC)
    &[20, 32],   // ADC OSR: set to 32 for 192KHz sample rate
    //&[61, 14],   // Select ADC processing block 14 (Filter C, Stereo, 5 biquads)
    &[27, 0x0C], // I2S, 16-bit, BCLK out, WCLK out, DOUT push-pull
    &[29, 0x00], // BCLK generated from DAC_MOD_CLK
    //&[29, 0x10], // Enable instead of above for DAC->ADC loopback
    &[30, (0x80 | 2)], // Divide BCLK (remove if we want more speed)
    // Power setup
    &[00, 0x01], // Select page 1
    &[02, 0x09], // Enable AVDD LDO
    &[01, 0x08], // Disable DVDD->AVDD connection
    &[02, 0x01], // Enable master analog power control
    //&[20, 0x2D], // De-pop settings
    &[10, 0x00], // Set 0.9V common-mode
    &[03, 0x00], // Left DAC in mode PTM_P3/4, class-AB driver
    &[04, 0x00], // Right DAC in mode PTM_P3/4, class-AB driver
    &[61, 0x00], // ADC in mode PTM_P3/4
    &[71, 0x32], // Set analog in power-up time to 3.1ms
    &[123, 0x01], // Slowly power up reference voltage (40ms)
    // ADC
    /*
    &[00, 0x01], // Select page 1
    &[52, 0x80], // Route IN1L to left MICPGA with 20K impedance
    &[54, 0x80], // Route Common mode 1 to left MICPGA negative input with 20K impedance
    // ^ Note: 10K and 40K also available, might be better
    &[55, 0x80], // Route IN1R to right MICPGA with 20K impedance
    &[57, 0x80], // Route Common mode 1 to right MICPGA negative input with 20K impedance
    &[59, 0x0C], // Left channel gain = 6dB
    &[60, 0x0C], // Right channel gain = 6dB
    // ^ Note: use this to adjust input gain!
    &[00, 0x00], // Select page 0
    &[81, 0xC1], // Power on ADC, enable gain soft-stepping
    &[82, 0x00], // Unmute ADCs
     */
    // DAC
    &[00, 0x01], // Select page 1
    &[12, 0x08], // Route left DAC to HPL
    &[13, 0x08], // Route right DAC to HPR
    &[14, 0x08], // Route left DAC to LOL
    &[15, 0x08], // Route right DAC to LOR
    &[16, 0x00], // Unmute HPL, 0dB gain
    &[17, 0x00], // Unmute HPR, 0dB gain
    &[18, 0x00], // Unmute LOL, 0dB gain
    &[19, 0x00], // Unmute LOR, 0dB gain
    &[09, 0x3C], // Power on headphone and line outputs
];

const INIT_CMD_2:&[&[u8]] = &[
    &[00, 0x00], // Select page 0
    &[65, 0x64], // 0dB left DAC digital gain
    &[66, 0x64], // 0dB right DAC digital gain
    // ^ Note: use this to adjust output gain! This works with soft-stepping!
    // ^ Left channel is set to control volume for both!
    &[63, 0xD5], // Enable DAC, route interface data to DAC, enable volume soft-stepping
    //&[63, 0xC1], // TEST: Enable DAC, but disable I2S connection
    //&[63, 0x01], // TEST: Disable DAC
    &[64, 0x02], // Unmute DAC, right channel volume is controlled by left channel control
    // ^ Note: automute can be configured here
];

pub fn tlv320_init<T: embedded_hal::blocking::i2c::Write>(
    i2c:&mut T, delay: &mut cortex_m::delay::Delay){
    // Initial reset
    i2c.write(0x18, &[0x00, 0x00]).debugless_unwrap();     // Select page 0
    i2c.write(0x18, &[0x01, 0x01]).debugless_unwrap();     // Software reset
    delay.delay_us(100);                     // Wait for reset to complete
    // Go through initialization command array
    for cmd in INIT_CMD_1{
        i2c.write(0x18, cmd).debugless_unwrap();
    }
    // Wait for anti-pop soft-stepping to complete before continuing
    //delay.delay_ms(3500);
    /*
    for cmd in INIT_CMD_2{
        i2c.write(0x18, cmd).debugless_unwrap();
    }
     */
}