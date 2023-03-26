//! # Panic
//! Custom panic handler for the CRONCH.
//! Blinks the Pi Pico's built-in LED to indicate the location of the panic.
//!
//! # Blink code
//! The blink code is a repeating sequence of four digits.
//! Digits are separated by a short pause, and repetitions of the entire code are separated by a long pause.
//! Each digit is a series of short blinks. For instance - one blink represents a one, and three blinks represent a three. One long blink represents a zero.
//!
//! ## Digits:
//! 1: File
//! 2: Line, hundreds place
//! 3: Line, tens place
//! 4: Line, ones place
//!
//! The 'file' value is an index into a list of source file names (see: FILES constant).
//! Indexing starts at 1. A value of 0 indicates an unknown source file.

use rp_pico::hal;
use hal::{
    pac,
};
use core::panic::PanicInfo;
use cortex_m::asm::delay;
use embedded_hal::digital::v2::OutputPin;

const FILES: &'static [&'static str] = &[
    "src/main.rs",
    "src/double_buf.rs",
    "src/init.rs",
    "src/tlv320.rs",
    "src/ui/mod.rs",
    "src/ui/expanders.rs",
    "src/ui/knob.rs",
    "src/ui/led_strip.rs",
];

const U: u32 = 45000;

macro_rules! blink_short {
    ($led_pin:ident) => {
        $led_pin.set_high().ok();
        delay(U * 300);
        $led_pin.set_low().ok();
        delay(U * 300);
    }
}

macro_rules! blink_long {
    ($led_pin:ident) => {
        $led_pin.set_high().ok();
        delay(U * 1500);
        $led_pin.set_low().ok();
        delay(U * 300);

    }
}

macro_rules! wait_short {
    () => {
        delay(U * 1500);
    }
}

macro_rules! wait_long {
    () => {
        delay(U * 3000);
    }
}

macro_rules! blink_num{
    ($led_pin:ident, $n:ident) => {
        if $n == 0 {
            blink_long!($led_pin);
        }
        else {
            for _ in 0..$n {
                blink_short!($led_pin);
            }
        }
        wait_short!();
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Steal all the peripherals
    //let mut core = unsafe { pac::CorePeripherals::steal() };
    let mut pac = unsafe { pac::Peripherals::steal() };

    // Init single-cycle IO
    let sio = hal::Sio::new(pac.SIO);

    // Init GPIO
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Disable all external peripherals
    let mut rst_pin = pins.gpio6.into_push_pull_output();
    rst_pin.set_low().ok();

    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().ok();

    if let Some(location) = info.location() {
        let mut line = location.line();
        let hundreds = line / 100;
        line -= 100 * hundreds;
        let tens = line / 10;
        line -= 10 * tens;
        let ones = line;

        let file_index: u32 = if let Some(i) = FILES.iter().position(|&x| x == location.file()) {
            i as u32 + 1
        }
        else {
            0
        };

        loop {
            blink_num!(led_pin, file_index);
            blink_num!(led_pin, hundreds);
            blink_num!(led_pin, tens);
            blink_num!(led_pin, ones);
            wait_long!();
        }
    }
    else {
        loop {
            led_pin.set_low().ok();
            delay(U * 100);
            led_pin.set_high().ok();
            delay(U * 100);
        }
    };

}