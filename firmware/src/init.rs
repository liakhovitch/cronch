/*
Init: a collection of macros to factor out init code and make main more readable.
Because init code is meant to be used only once in a very specific context, I believe that macros
may actually be the idiomatically correct tool for this.
 */

#[macro_export]
macro_rules! init_clocks {
    ($pac:ident, $watchdog:ident) => {{
        hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            $pac.XOSC,
            $pac.CLOCKS,
            $pac.PLL_SYS,
            $pac.PLL_USB,
            &mut $pac.RESETS,
            &mut $watchdog,
        ).ok()
    }}
}

#[macro_export]
macro_rules! init_audio_clk {
    ($pwm_slices:ident, $pins:ident) => {
        // Configure audio chip clock signal
        let pwm = &mut $pwm_slices.pwm0;
        pwm.clr_ph_correct();
        pwm.set_top(9);
        pwm.enable();
        let mclk = &mut pwm.channel_a;
        mclk.set_duty(5);
        mclk.output_to($pins.gpio16);
    }
}

#[macro_export]
macro_rules! init_ui_i2c {
    ($pins:ident, $pac:ident, $clocks:ident) => {{
        let sda_pin = $pins.gpio2.into_mode::<hal::gpio::FunctionI2C>();
        let scl_pin = $pins.gpio3.into_mode::<hal::gpio::FunctionI2C>();

        hal::I2C::i2c1(
            $pac.I2C1,
            sda_pin,
            scl_pin,
            400.kHz(),
            &mut $pac.RESETS,
            &$clocks.system_clock,
        )
    }}
}

#[macro_export]
macro_rules! init_audio_i2c {
    ($pins:ident, $pac:ident, $clocks:ident) => {{
    // Extra scope turns this entire block into one expression, needed for ret value to work properly

        let sda_pin = $pins.gpio4.into_mode::<hal::gpio::FunctionI2C>();
        let scl_pin = $pins.gpio5.into_mode::<hal::gpio::FunctionI2C>();

        // Return value: I2C object
        hal::I2C::i2c0(
            $pac.I2C0,
            sda_pin,
            scl_pin,
            400.kHz(),
            &mut $pac.RESETS,
            &$clocks.system_clock,
        )
    }}
}

#[macro_export]
macro_rules! init_psram {
    ($pins:ident, $pac:ident, $clocks:ident, $delay:ident) => {{
        let _spi_sclk = $pins.gpio8.into_mode::<hal::gpio::FunctionSpi>();
        let _spi_mosi = $pins.gpio10.into_mode::<hal::gpio::FunctionSpi>();
        let _spi_miso = $pins.gpio11.into_mode::<hal::gpio::FunctionSpi>();
        let mut cs = $pins.gpio9.into_push_pull_output();
        let spi = hal::Spi::<_, _, 8>::new($pac.SPI1);
        // Initialize SPI
        let mut spi = spi.init(
            &mut $pac.RESETS,
            $clocks.peripheral_clock.freq(),
            64.MHz(),
            &embedded_hal::spi::MODE_0,
        );
        cs.set_high().unwrap();
        $delay.delay_ms(1);
        (spi, cs)
    }}
}