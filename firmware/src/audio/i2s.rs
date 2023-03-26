use rp2040_hal::gpio::{Disabled, DisabledConfig, Function, FunctionConfig, Pin, PinId, ValidPinMode};
use rp2040_hal::pio::{PIO, PIOExt, Rx, ShiftDirection, StateMachine, StateMachineIndex, Tx, UninitStateMachine};
use rp_pico::hal;

pub struct I2S<'pio, P, SMI, BCLK, WCLK, DIN, DOUT>
where
    P: PIOExt + FunctionConfig,
    SMI: StateMachineIndex,
    BCLK: PinId,
    WCLK: PinId,
    DIN: PinId,
    DOUT: PinId,
    Function<P>: ValidPinMode<BCLK> + ValidPinMode<WCLK> + ValidPinMode<DIN> + ValidPinMode<DOUT>,
{
    _pio: &'pio mut PIO<P>,
    _sm: StateMachine<(P, SMI), rp2040_hal::pio::Running>,
    tx: Tx<(P, SMI)>,
    rx: Rx<(P, SMI)>,
    _bclk: Pin<BCLK, Function<P>>,
    _wclk: Pin<WCLK, Function<P>>,
    _din: Pin<DIN, Function<P>>,
    _dout: Pin<DOUT, Function<P>>,
}

impl<'pio, P, SMI, BCLK, WCLK, DIN, DOUT> I2S<'pio, P, SMI, BCLK, WCLK, DIN, DOUT>
where
    P: PIOExt + FunctionConfig,
    SMI: StateMachineIndex,
    BCLK: PinId,
    WCLK: PinId,
    DIN: PinId,
    DOUT: PinId,
    Function<P>: ValidPinMode<BCLK> + ValidPinMode<WCLK> + ValidPinMode<DIN> + ValidPinMode<DOUT>,
{
    pub fn new<
        BclkDisabledConfig: DisabledConfig,
        WclkDisabledConfig: DisabledConfig,
        DinDisabledConfig: DisabledConfig,
        DoutDisabledConfig: DisabledConfig,
    >
    (
        pio: &'pio mut PIO<P>,
        bclk: rp2040_hal::gpio::Pin<BCLK, Disabled<WclkDisabledConfig>>,
        wclk: rp2040_hal::gpio::Pin<WCLK, Disabled<BclkDisabledConfig>>,
        din: rp2040_hal::gpio::Pin<DIN, Disabled<DinDisabledConfig>>,
        dout: rp2040_hal::gpio::Pin<DOUT, Disabled<DoutDisabledConfig>>,
        sm: UninitStateMachine<(P, SMI)>,
    ) -> Self
    where
        Function<P>: ValidPinMode<BCLK> + ValidPinMode<WCLK> + ValidPinMode<DIN> + ValidPinMode<DOUT>,
    {
        let _bclk: Pin<_, Function<P>> = bclk.into_mode();
        let _wclk: Pin<_, Function<P>> = wclk.into_mode();
        let _din: Pin<_, Function<P>> = din.into_mode();
        let _dout: Pin<_, Function<P>> = dout.into_mode();

        let mut program = pio_proc::pio_asm!(
            "pull"                  // 0  Pull first word into OSR
            "wait 0 gpio 0"         // 1  Wait for low edge on WCLK before starting
            ".wrap_target"          //    Main loop
            "left_loop:"            //    Left channel loop
            "wait 1 gpio 0"         // 2  Wait for high on WCLK (indicates left channel)
            "out pins 1"            // 3  Output one bit from OSR
            "wait 1 gpio 0"         // 4  Wait for high bit clock
            "in pins 1"             // 5  Read one bit to ISR
            "wait 0 gpio 0"         // 6  Wait for low bit clock
            "jmp !osre left_loop"   // 7  If OSR still has data, jump to left_loop
            "pull"                  // 8  Pull another word into OSR
            "right_loop:"
            "wait 0 gpio 0"         // 9
            "out pins 1"            // 10
            "wait 1 gpio 0"         // 11
            "in pins 1"             // 12
            "wait 0 gpio 0"         // 13
            "jmp !osre right_loop"  // 14
            "pull"                  // 15
            ".wrap"
        ).program;

        // Patch WCLK and BCLK pin numbers
        program.code[1] |= u16::from(WCLK::DYN.num);
        program.code[2] |= u16::from(WCLK::DYN.num);
        program.code[9] |= u16::from(WCLK::DYN.num);
        program.code[4] |= u16::from(BCLK::DYN.num);
        program.code[6] |= u16::from(BCLK::DYN.num);
        program.code[11] |= u16::from(BCLK::DYN.num);
        program.code[13] |= u16::from(BCLK::DYN.num);

        // Initialize and start PIO
        let installed = pio.install(&program).unwrap();
        let (int, frac) = (1, 0); // Run PIO at full speed
        let (mut sm, rx, mut tx) = rp2040_hal::pio::PIOBuilder::from_program(installed)
            .buffers(rp2040_hal::pio::Buffers::RxTx)
            .out_pins(DOUT::DYN.num, 1)
            .in_pin_base(DIN::DYN.num)
            // OSR config
            .out_shift_direction(rp2040_hal::pio::ShiftDirection::Left)
            .autopull(false)
            .pull_threshold(32)
            // ISR config
            .in_shift_direction(ShiftDirection::Left)
            .autopush(true)
            .push_threshold(32)
            .clock_divisor_fixed_point(int, frac)
            .build(sm);

        sm.set_pindirs([(BCLK::DYN.num, hal::pio::PinDir::Input)]);
        sm.set_pindirs([(WCLK::DYN.num, hal::pio::PinDir::Input)]);
        sm.set_pindirs([(DIN::DYN.num, hal::pio::PinDir::Output)]);
        sm.set_pindirs([(DOUT::DYN.num, hal::pio::PinDir::Input)]);

        // Write initial sample
        tx.write(0);
        tx.write(0);

        // Kick off the state machine
        let sm = sm.start();

        // Construct and return driver object
        I2S{
            _pio: pio,
            _sm: sm,
            tx,
            rx,
            _bclk,
            _wclk,
            _din,
            _dout,
        }
    }

    pub fn rw(&mut self, sample: (i32, i32)) -> (i32, i32){
        self.tx.write(u32::from_le_bytes(sample.0.to_le_bytes()));
        self.tx.write(u32::from_le_bytes(sample.1.to_le_bytes()));
        // Wait for driver to return a sample
        let left: i32 = loop{
            if let Some(n) = self.rx.read(){
                break i32::from_le_bytes(n.to_le_bytes())
            }
        };
        let right: i32 = loop{
            if let Some(n) = self.rx.read(){
                break i32::from_le_bytes(n.to_le_bytes())
            }
        };
        (left, right)
    }
}