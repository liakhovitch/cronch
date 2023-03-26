#![crate_name = "cronch"]

//! # CRONCH
//! By Anton Liakhovitch

#![no_std]
#![no_main]

mod double_buf;
mod ui;
mod audio;
mod init;
mod panic;

use cortex_m::singleton;

use double_buf::{DoubleBuf, DoubleBufPort};

use rp_pico::entry;

// GPIO traits
use embedded_hal::{
    digital::v2::OutputPin,
    PwmPin,
};

use embedded_hal::prelude::*;

use fugit::RateExtU32;

use rp_pico::hal;
use hal::{
    prelude::*,
    adc::Adc,
    multicore::{Multicore, Stack},
    pac,
};

use audio::tlv320::init_tlv320;
use ui::{expanders, led_strip, knob};
use crate::audio::i2s;

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
    let _timer = rp2040_hal::timer::Timer::new(pac.TIMER , &mut pac.RESETS);

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
    let intercore = singleton!(:DoubleBuf::<ui::UiOutput, ui::UiInput, 0, 1> = unsafe{
        DoubleBuf::new(||{Default::default()}, ||{Default::default()})
    }).unwrap();
    let (mut intercore_ui, mut intercore_audio) = intercore.split().unwrap();

    #[cfg(not(feature="headless"))]
    {
        // Init ADC
        let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
        let mut mix_knob = knob::Knob::new(pins.gpio26.into_floating_input(), 4);
        let mut clk_knob = knob::Knob::new(pins.gpio27.into_floating_input(), 4);
        let mut fdbk_knob = knob::Knob::new(pins.gpio28.into_floating_input(), 4);

        // Init UI I2C
        let mut ui_i2c = init_ui_i2c!(pins, pac, clocks);

        // Setup core 1
        let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
        mc.cores()[1].spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            // -- ALL CODE IN THIS SCOPE RUNS ON CORE1, IN PARALLEL WITH CORE0 --

            // Acquire core peripherals for Core 1
            let _core = unsafe { pac::CorePeripherals::steal() };
            // SIO peripherals are also duplicated for each core so stealing them is OK
            let sio = hal::Sio::new(unsafe { pac::Peripherals::steal().SIO });
            // Init 64-bit timebase timer
            let timer = rp2040_hal::timer::Timer::new(unsafe { pac::Peripherals::steal().TIMER }, &mut unsafe { pac::Peripherals::steal().RESETS });

            // Init LED strip
            let mut led_strip = led_strip::LedStrip::new(&mut ui_i2c, &timer, &sio.hwdivider, 800, 800).unwrap();

            // Init front panel IO expanders
            let expanders = expanders::Expanders::new(&mut ui_i2c).unwrap();

            let mut out: ui::UiOutput = Default::default();
            let mut input: ui::UiInput = Default::default();
            // Read the ADCs to stabilize the hysteresis state
            if let Some(n) = fdbk_knob.read(&mut adc).unwrap() { out.fdbk_knob = n };
            if let Some(n) = clk_knob.read(&mut adc).unwrap() { out.clk_knob = n };
            if let Some(n) = mix_knob.read(&mut adc).unwrap() { out.mix_knob = n };
            loop {
                led_strip.update(&mut ui_i2c, &out, &input).unwrap();

                read_panel!(out, ui_i2c, expanders, adc, fdbk_knob, clk_knob, mix_knob, led_strip);

                intercore_ui.rw(|w| { *w = out; }, |r| { input = *r; });
            }
        }).unwrap();

    } // End conditional compilation

    // Init TLV320AIC3254 MCLK signal
    init_audio_clk!(pwm_slices, pins);

    // Init audio I2C
    let mut audio_i2c = init_audio_i2c!(pins, pac, clocks);

    init_tlv320(&mut audio_i2c, &mut delay);

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

    let mut out: ui::UiInput = Default::default();
    let mut input: ui::UiOutput = Default::default();

    let mut sample = (0i32, 0i32);

    let (mut pio, sm0, sm1, sm2, sm3) = pac.PIO0.split(&mut pac.RESETS);
    let mut i2s = i2s::I2S::new(&mut pio, pins.gpio17, pins.gpio18, pins.gpio19, pins.gpio20, sm0, sm1, sm2, sm3);
    // Turn off PIO input synchronizers. This is necessary to allow PIO inputs to run at full speed so we can support 192KHz 32bit audio
    unsafe{ pac::Peripherals::steal().PIO0.input_sync_bypass.write(|w| w.bits(0xFFFFFFFF)); }

    loop {

        i2s.write_left(sample.0);
        i2s.write_right(sample.1);
        sample.0 = i2s.read_left();
        sample.1 = i2s.read_right();

        out.write_addr = out.write_addr.wrapping_add(1);
        out.read_addr = out.write_addr;
        intercore_audio.rw(|w|{ *w = out; }, |r|{ input = *r; });

        cortex_m::asm::delay(360); // This represents the remaining CPU budget
        // Note: the actual available budget is the delay value x1.5 since asm::delay takes
        // 1.5 * (input) cycles on Cortex-M0.
        // The theoretical maximum budget is:
        // (64 bits/sample) * (125MHz sysclk / 12.5MHz I2S clock) = 640 CPU cycles per sample
    }
}

// End of file
