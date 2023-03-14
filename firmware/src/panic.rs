use rp_pico::hal;
use hal::{
    prelude::*,
    pac,
};
use core::panic::PanicInfo;
use cortex_m::asm::delay;
use embedded_hal::digital::v2::OutputPin;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Steal all the peripherals
    let mut core = unsafe { pac::CorePeripherals::steal() };
    let mut pac = unsafe { pac::Peripherals::steal() };
    // Which CPU are we?
    if pac.SIO.cpuid.read().bits() == 0 {
        // If we are CPU0, kill CPU1
        pac.PSM.frce_off.write(|w|{w.proc1().set_bit()});
    }
    else {
        // If we are CPU1, kill CPU0
        pac.PSM.frce_off.write(|w|{w.proc0().set_bit()});
    }

    // Init single-cycle IO
    let mut sio = hal::Sio::new(pac.SIO);

    // Init GPIO
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Disable all external peripherals
    let mut rst_pin = pins.gpio6.into_push_pull_output();
    let _ = rst_pin.set_low();

    let mut led_pin = pins.led.into_push_pull_output();
    let _ = led_pin.set_high();

    loop {
        let _ = led_pin.set_low();
        delay(125_000_000);
        let _ = led_pin.set_high();
        delay(125_000_000);
    }
}