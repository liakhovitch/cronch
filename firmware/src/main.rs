#![crate_name = "cronch"]

//! # CRONCH
//! By Anton Liakhovitch

#![no_std]
#![no_main]

pub mod double_buf;
pub mod tlv320;
pub mod ui;

use cortex_m::asm::nop;
use cortex_m::singleton;

use double_buf::{DoubleBuf, DoubleBufPort};

use rp_pico::entry;

// GPIO traits
use embedded_hal::{
    digital::v2::OutputPin,
    blocking::i2c::Write,
    PwmPin,
    adc::OneShot,
};

use embedded_hal::prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_spi_Transfer};

use fugit::RateExtU32;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

use rp_pico::hal;
use hal::{
    prelude::*,
    gpio::{Pin},
    adc::Adc,
    multicore::{Multicore, Stack},
    pac,
};

use tlv320::tlv320_init;

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let mut sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );



    // Init PWMs
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Set up global peripheral reset and reset all external peripherals
    let mut rst_pin = pins.gpio6.into_push_pull_output();
    rst_pin.set_low().unwrap();
    delay.delay_ms(5);
    rst_pin.set_high().unwrap();
    delay.delay_ms(5);

    // Configure audio chip clock signal
    let pwm = &mut pwm_slices.pwm0;
    pwm.clr_ph_correct();
    pwm.set_top(9);
    pwm.enable();
    let mclk = &mut pwm.channel_a;
    mclk.set_duty(5);
    mclk.output_to(pins.gpio16);

    // Configure audio control interface (I2C)
    let sda_pin = pins.gpio4.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio5.into_mode::<hal::gpio::FunctionI2C>();
    let mut din = pins.gpio19.into_push_pull_output();
    din.set_low().unwrap();
    let mut audio_i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    // Create a statically allocated double buffer
    let foo = singleton!(:DoubleBuf::<(u32, u32), (u32, u32), 0, 1> = unsafe{
        DoubleBuf::new(||{(0,0)}, ||{(0,0)})
    }).unwrap();
    let (mut intercore_ui, mut intercore_audio) = foo.split().unwrap();

    // Init ADC
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut mix_knob_pin = pins.gpio26.into_floating_input();
    let mut clk_knob_pin = pins.gpio27.into_floating_input();
    let mut fdbk_knob_pin = pins.gpio28.into_floating_input();

    // "UI" I2C config
    let sda_pin = pins.gpio2.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio3.into_mode::<hal::gpio::FunctionI2C>();
    let mut ui_i2c = hal::I2C::i2c1(
        pac.I2C1,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    // Setup core 1
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            let core = unsafe { pac::CorePeripherals::steal() };
            let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
            // Initial setup for LED strip
            ui_i2c.write(0x60, &[
                0x00, 0x00,
                0x0D, 0b00001000,
                0x0F, 0x00,
                0x0C, 0x00,
            ]).unwrap();

            // Initial setup for expander 3
            ui_i2c.write(0x23, &[0x06, 0xFF]).unwrap(); // Turn off LEDs
            ui_i2c.write(0x23, &[0x5C, 0b00000100]).unwrap(); // Open-drain output
            ui_i2c.write(0x23, &[0x0C, 0xFF, 0xFF, 0x00]).unwrap(); // DDR
            ui_i2c.write(0x23, &[0x08, 0xFF, 0xFF, 0x00]).unwrap(); // Invert inputs
            // Minimum drive strength
            ui_i2c.write(0x23, &[0x40, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x0FF]).unwrap();
            ui_i2c.write(0x23, &[0x4C, 0xFF, 0xFF, 0x00]).unwrap(); // Enable pull-ups

            let mut vals_read = (0u32, 0u32);

            loop {
                let mut buf: [u8; 2] = [0, 0];
                ui_i2c.write(0x23, &[0x00]).unwrap();
                ui_i2c.read(0x23, &mut buf).unwrap();
                let ledbyte: u8 = match buf[1] | 0b11110001 {
                    0b11110001 => 0b11111110,
                    0b11110011 => 0b11111101,
                    0b11110101 => 0b11111011,
                    _          => 0b11110111,
                } & match buf[1] | 0b00011111 {
                    0b00011111 => 0b11101111,
                    0b00111111 => 0b11011111,
                    0b01011111 => 0b10111111,
                    _          => 0b01111111,
                } | match buf[1] | 0b011101111 {
                    0b11111111 => 0b00001111,
                    _          => 0b00000000,
                } | match buf[1] | 0b11111110 {
                    0b11111111 => 0b11110000,
                    _          => 0b00000000,
                };
                ui_i2c.write(0x23, &[0x06, ledbyte]).unwrap();

                let mut reading_avg: u32 = 0;
                for _ in 0..32 {
                    let reading: u16 = adc.read(&mut mix_knob_pin).unwrap();
                    reading_avg += reading as u32;
                }
                reading_avg >>= 5;
                let shift: u32 = (reading_avg >> 6) as u32;
                let mix_disp: u32 = if shift < 32{
                    0xFFFFFFFF >> (31 - shift)
                } else {
                    0xFFFFFFFF << shift
                };
                let mut reading_avg: u32 = 0;
                for _ in 0..32 {
                    let reading: u16 = adc.read(&mut clk_knob_pin).unwrap();
                    reading_avg += reading as u32;
                }
                reading_avg >>= 5;
                let shift: u32 = (reading_avg >> 7) as u32;
                let clk_disp: u32 = 0x01 << shift;

                let (mix_disp_r, clk_disp_r) = vals_read;
                ui_i2c.write(0x60, &[
                    1, mix_disp_r.to_le_bytes()[0],
                    2, mix_disp_r.to_le_bytes()[1],
                    3, mix_disp_r.to_le_bytes()[2],
                    4, mix_disp_r.to_le_bytes()[3],
                    5, clk_disp_r.to_le_bytes()[0],
                    6, clk_disp_r.to_le_bytes()[1],
                    7, clk_disp_r.to_le_bytes()[2],
                    8, clk_disp_r.to_le_bytes()[3],
                    0x0c, 0x00
                ]).unwrap();
                ui_i2c.write(0x60, &[0x0C, 0x00]).unwrap();

                intercore_ui.rw(|x|{*x = (mix_disp, clk_disp)}, |x|{vals_read = *x});
            }
        })
        .unwrap();
    //tlv320_init(&mut audio_i2c, &mut delay);

    // PSRAM config
    let _spi_sclk = pins.gpio8.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio10.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_miso = pins.gpio11.into_mode::<hal::gpio::FunctionSpi>();
    let mut spi_cs = pins.gpio9.into_push_pull_output();
    let spi = hal::Spi::<_, _, 8>::new(pac.SPI1);
    // Initialize SPI
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        64.MHz(),
        &embedded_hal::spi::MODE_0,
    );
    spi_cs.set_high().unwrap();
    delay.delay_ms(1);

    // Write to SPI
    let addr: u32 = 0x05;
    let addr_bytes = addr.to_le_bytes();
    let mut buffer: [u8; 5] = [0x02, addr_bytes[2], addr_bytes[1], addr_bytes[0], 0x22];
    spi_cs.set_low().unwrap();
    spi.transfer(&mut buffer).unwrap();
    spi_cs.set_high().unwrap();
    delay.delay_ms(1);

    let mut buffer: [u8; 6] = [0x0B, addr_bytes[2], addr_bytes[1], addr_bytes[0], 0x00, 0x00];
    spi_cs.set_low().unwrap();
    spi.transfer(&mut buffer).unwrap();
    spi_cs.set_high().unwrap();


    // Set the LED to be an output
    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();


    let mut save = (0u32, 0u32);

    loop {
        /*
        led_pin.set_high().unwrap();
        delay.delay_ms(2000);
        led_pin.set_low().unwrap();
        delay.delay_ms(2000);
         */

        let write = save;
        intercore_audio.rw(|x|{*x = write}, |x|{save = *x});
    }
}

// End of file
