use rp2040_hal::gpio::{Disabled, DisabledConfig, Function, FunctionConfig, Pin, PinId, ValidPinMode};
use rp2040_hal::pio::{PIO, PIOExt, Running, Rx, ShiftDirection, SM0, SM1, SM2, SM3, StateMachineGroup4, Tx, UninitStateMachine};
use rp_pico::hal;

pub struct I2S<'pio, P, BCLK, WCLK, DIN, DOUT>
where
    P: PIOExt + FunctionConfig,
    BCLK: PinId,
    WCLK: PinId,
    DIN: PinId,
    DOUT: PinId,
    Function<P>: ValidPinMode<BCLK> + ValidPinMode<WCLK> + ValidPinMode<DIN> + ValidPinMode<DOUT>,
{
    _pio: &'pio mut PIO<P>,
    _group: StateMachineGroup4<P, SM0, SM1, SM2, SM3, Running>,
    l_tx: Tx<(P, SM0)>,
    r_tx: Tx<(P, SM1)>,
    l_rx: Rx<(P, SM2)>,
    r_rx: Rx<(P, SM3)>,
    _bclk: Pin<BCLK, Function<P>>,
    _wclk: Pin<WCLK, Function<P>>,
    _din: Pin<DIN, Function<P>>,
    _dout: Pin<DOUT, Function<P>>,
}

impl<'pio, P, BCLK, WCLK, DIN, DOUT> I2S<'pio, P, BCLK, WCLK, DIN, DOUT>
where
    P: PIOExt + FunctionConfig,
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
        sm0: UninitStateMachine<(P, SM0)>,
        sm1: UninitStateMachine<(P, SM1)>,
        sm2: UninitStateMachine<(P, SM2)>,
        sm3: UninitStateMachine<(P, SM3)>,
    ) -> Self
    where
        Function<P>: ValidPinMode<BCLK> + ValidPinMode<WCLK> + ValidPinMode<DIN> + ValidPinMode<DOUT>,
    {
        let _bclk: Pin<_, Function<P>> = bclk.into_mode();
        let _wclk: Pin<_, Function<P>> = wclk.into_mode();
        let _din: Pin<_, Function<P>> = din.into_mode();
        let _dout: Pin<_, Function<P>> = dout.into_mode();

        let mut prog_ltx = pio_proc::pio_asm!(
            ".wrap_target"
            "pull"
            "wait 0 gpio 0"//WCLK
            "wait 1 gpio 0"//WCLK
            "loop:"
            "out pins 1"
            "wait 1 gpio 0"//BCLK
            "wait 0 gpio 0"//BCLK
            "jmp !osre loop"
            ".wrap"
        ).program;

        // Patch WCLK and BCLK pin numbers
        prog_ltx.code[1] |= u16::from(WCLK::DYN.num);
        prog_ltx.code[2] |= u16::from(WCLK::DYN.num);
        prog_ltx.code[4] |= u16::from(BCLK::DYN.num);
        prog_ltx.code[5] |= u16::from(BCLK::DYN.num);

        let mut prog_rtx = pio_proc::pio_asm!(
            "wait 0 gpio 0"//WCLK
            ".wrap_target"
            "pull"
            "wait 1 gpio 0"//WCLK
            "wait 0 gpio 0"//WCLK
            "loop:"
            "out pins 1"
            "wait 1 gpio 0"//BCLK
            "wait 0 gpio 0"//BCLK
            "jmp !osre loop"
            ".wrap"
        ).program;

        // Patch WCLK and BCLK pin numbers
        prog_rtx.code[0] |= u16::from(WCLK::DYN.num);
        prog_rtx.code[2] |= u16::from(WCLK::DYN.num);
        prog_rtx.code[3] |= u16::from(WCLK::DYN.num);
        prog_rtx.code[5] |= u16::from(BCLK::DYN.num);
        prog_rtx.code[6] |= u16::from(BCLK::DYN.num);

        let mut prog_lrx = pio_proc::pio_asm!(
            ".wrap_target"
            "set x 31"
            "wait 0 gpio 0"//WCLK
            "wait 1 gpio 0"//WCLK
            "loop:"
            "wait 1 gpio 0"//BCLK
            "in pins 1"
            "wait 0 gpio 0"//BCLK
            "jmp x-- loop"
            ".wrap"
        ).program;

        // Patch WCLK and BCLK pin numbers
        prog_lrx.code[1] |= u16::from(WCLK::DYN.num);
        prog_lrx.code[2] |= u16::from(WCLK::DYN.num);
        prog_lrx.code[3] |= u16::from(BCLK::DYN.num);
        prog_lrx.code[5] |= u16::from(BCLK::DYN.num);

        let mut prog_rrx = pio_proc::pio_asm!(
            "wait 0 gpio 0"//WCLK
            ".wrap_target"
            "set x 31"
            "wait 1 gpio 0"//WCLK
            "wait 0 gpio 0"//WCLK
            "loop:"
            "wait 1 gpio 0"//BCLK
            "in pins 1"
            "wait 0 gpio 0"//BCLK
            "jmp x-- loop"
            ".wrap"
        ).program;

        // Patch WCLK and BCLK pin numbers
        prog_rrx.code[0] |= u16::from(WCLK::DYN.num);
        prog_rrx.code[2] |= u16::from(WCLK::DYN.num);
        prog_rrx.code[3] |= u16::from(WCLK::DYN.num);
        prog_rrx.code[4] |= u16::from(BCLK::DYN.num);
        prog_rrx.code[6] |= u16::from(BCLK::DYN.num);

        let installed_ltx = pio.install(&prog_ltx).unwrap();
        let installed_rtx = pio.install(&prog_rtx).unwrap();
        let installed_lrx = pio.install(&prog_lrx).unwrap();
        let installed_rrx = pio.install(&prog_rrx).unwrap();

        let (mut sm0, _, mut l_tx) = rp2040_hal::pio::PIOBuilder::from_program(installed_ltx)
            .buffers(rp2040_hal::pio::Buffers::OnlyTx)
            .out_pins(DIN::DYN.num, 1)
            // OSR config
            .out_shift_direction(rp2040_hal::pio::ShiftDirection::Left)
            .autopull(false)
            .pull_threshold(32)
            .build(sm0);

        let (sm1, _, mut r_tx) = rp2040_hal::pio::PIOBuilder::from_program(installed_rtx)
            .buffers(rp2040_hal::pio::Buffers::OnlyTx)
            .out_pins(DIN::DYN.num, 1)
            .set_pins(DIN::DYN.num, 1)
            // OSR config
            .out_shift_direction(rp2040_hal::pio::ShiftDirection::Left)
            .autopull(false)
            .pull_threshold(32)
            .build(sm1);

        let (sm2, l_rx, _) = rp2040_hal::pio::PIOBuilder::from_program(installed_lrx)
            .buffers(rp2040_hal::pio::Buffers::OnlyRx)
            .in_pin_base(DOUT::DYN.num)
            // ISR config
            .in_shift_direction(ShiftDirection::Left)
            .autopush(true)
            .push_threshold(32)
            .build(sm2);

        let (sm3, r_rx, _) = rp2040_hal::pio::PIOBuilder::from_program(installed_rrx)
            .buffers(rp2040_hal::pio::Buffers::OnlyRx)
            .in_pin_base(DOUT::DYN.num)
            // ISR config
            .in_shift_direction(ShiftDirection::Left)
            .autopush(true)
            .push_threshold(32)
            .build(sm3);

        sm0.set_pindirs([(BCLK::DYN.num, hal::pio::PinDir::Input)]);
        sm0.set_pindirs([(WCLK::DYN.num, hal::pio::PinDir::Input)]);
        sm0.set_pindirs([(DIN::DYN.num, hal::pio::PinDir::Output)]);
        sm0.set_pindirs([(DOUT::DYN.num, hal::pio::PinDir::Input)]);

        // Kick off the state machine
        let group = sm0.with(sm1).with(sm2).with(sm3).sync().start();

        // Construct and return driver object
        I2S{
            _pio: pio,
            _group: group,
            l_tx,
            r_tx,
            l_rx,
            r_rx,
            _bclk,
            _wclk,
            _din,
            _dout,
        }
    }

    pub fn read_left(&mut self) -> i32{
        loop{
            if let Some(n) = self.l_rx.read(){
                break i32::from_le_bytes(n.to_le_bytes())
            }
        }
    }

    pub fn read_right(&mut self) -> i32{
        loop{
            if let Some(n) = self.r_rx.read(){
                break i32::from_le_bytes(n.to_le_bytes())
            }
        }
    }

    pub fn write_left(&mut self, sample: i32){
        self.l_tx.write(u32::from_le_bytes(sample.to_le_bytes()));
    }

    pub fn write_right(&mut self, sample: i32){
        self.r_tx.write(u32::from_le_bytes(sample.to_le_bytes()));
    }

}