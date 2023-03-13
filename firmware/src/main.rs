#![crate_name = "cronch"]

//! # CRONCH
//! By Anton Liakhovitch

#![no_std]
#![no_main]

pub mod double_buf;
pub mod tlv320;
pub mod ui;
#[macro_use]
pub mod init;

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

use tlv320::init_tlv320;
use ui::*;

// Reserve memory for Core 1's stack
// (Stack memory for Core 0 is reserved automatically)
static mut CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
    // Acquire ownership of all peripherals
    let mut pac = pac::Peripherals::take().unwrap();
    // Acquire ownership of core peripherals
    let core = pac::CorePeripherals::take().unwrap();

    // Init watchdog
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Init clocks with a 125MHz system clock
    let clocks = init_clocks!(pac, watchdog).unwrap();

    // Init systick timer and delay
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Init single-cycle IO
    let mut sio = hal::Sio::new(pac.SIO);

    // Init GPIO
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

    // Create a statically allocated double buffer to share data between cores
    let intercore = singleton!(:DoubleBuf::<UiOutput, UiInput, 0, 1> = unsafe{
        DoubleBuf::new(||{Default::default()}, ||{Default::default()})
    }).unwrap();
    let (mut intercore_ui, mut intercore_audio) = intercore.split().unwrap();

    // Init ADC
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut mix_knob_pin = pins.gpio26.into_floating_input();
    let mut clk_knob_pin = pins.gpio27.into_floating_input();
    let mut fdbk_knob_pin = pins.gpio28.into_floating_input();

    // Init UI I2C
    let mut ui_i2c = init_ui_i2c!(pins, pac, clocks);

    // Setup core 1
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let core_freq = clocks.system_clock.freq().to_Hz();
    mc.cores()[1].spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        // Acquire core peripherals for Core 1
        let core = unsafe { pac::CorePeripherals::steal() };
        let delay = cortex_m::delay::Delay::new(core.SYST, core_freq);

        // Init LED strip
        let led_strip = LedStrip::new(&mut ui_i2c).unwrap();

        // Init front panel IO expanders
        let expanders = Expanders::new(&mut ui_i2c).unwrap();

        loop {

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

            //let (mix_disp_r, clk_disp_r) = vals_read;
            //led_strip.write(&mut ui_i2c, clk_disp_r, mix_disp_r).unwrap();

            intercore_ui.rw(|w|{
                expanders.read_opsel_en(&mut ui_i2c, &mut w.op1, &mut w.op2, &mut w.op1_en, &mut w.op2_en).unwrap();
                expanders.write_opsel_leds(&mut ui_i2c, &w.op1, &w.op2, &w.op1_en, &w.op2_en).unwrap();
            }, |r|{
                let red: u16 = 1 << (r.write_addr >> 11);
                let green: u16 = 1 << (r.read_addr >> 11);
            });
        }
    }).unwrap();

    // Init TLV320AIC3254 MCLK signal
    init_audio_clk!(pwm_slices, pins);

    // Init audio I2C
    let mut audio_i2c = init_audio_i2c!(pins, pac, clocks);
    let mut din = pins.gpio19.into_push_pull_output();
    din.set_low().unwrap();

    //init_tlv320(&mut audio_i2c, &mut delay);

    // Init PSRAM
    let (mut psram_spi, mut psram_cs) = init_psram!(pins, pac, clocks, delay);

    // Write to SPI
    let addr: u32 = 0x05;
    let addr_bytes = addr.to_le_bytes();
    let mut buffer: [u8; 5] = [0x02, addr_bytes[2], addr_bytes[1], addr_bytes[0], 0x22];
    psram_cs.set_low().unwrap();
    psram_spi.transfer(&mut buffer).unwrap();
    psram_cs.set_high().unwrap();
    delay.delay_ms(1);

    let mut buffer: [u8; 6] = [0x0B, addr_bytes[2], addr_bytes[1], addr_bytes[0], 0x00, 0x00];
    psram_cs.set_low().unwrap();
    psram_spi.transfer(&mut buffer).unwrap();
    psram_cs.set_high().unwrap();


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
        //intercore_audio.rw(|x|{*x = write}, |x|{save = *x});
    }
}

// End of file
