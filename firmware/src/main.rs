#![crate_name = "cronch"]

//! # CRONCH
//! By Anton Liakhovitch

#![no_std]
#![no_main]

mod double_buf;
mod tlv320;
mod ui;
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

//use panic_halt as _;

use rp_pico::hal;
use hal::{
    prelude::*,
    adc::Adc,
    multicore::{Multicore, Stack},
    pac,
};

use tlv320::init_tlv320;
use ui::{expanders, led_strip, knob};

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
    let timer = rp2040_hal::timer::Timer::new(pac.TIMER , &mut pac.RESETS);

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
        let sio = hal::Sio::new(unsafe{ pac::Peripherals::steal().SIO });
        // Init 64-bit timebase timer
        let timer = rp2040_hal::timer::Timer::new(unsafe{ pac::Peripherals::steal().TIMER }, &mut unsafe{ pac::Peripherals::steal().RESETS });

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

            intercore_ui.rw(|w|{ *w = out; }, |r|{ input = *r; });
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

    let mut out: ui::UiInput = Default::default();
    let mut input: ui::UiOutput = Default::default();

    loop {
        /*
        led_pin.set_high().unwrap();
        delay.delay_ms(2000);
        led_pin.set_low().unwrap();
        delay.delay_ms(2000);
         */
        delay.delay_ms(10);
        out.write_addr = input.op1_arg << 11;
        out.read_addr = input.op2_arg << 11;
        intercore_audio.rw(|w|{ *w = out; }, |r|{ input = *r; });
    }
}

// End of file
