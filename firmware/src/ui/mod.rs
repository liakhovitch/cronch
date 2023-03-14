pub mod knob;
pub mod led_strip;
pub mod expanders;

#[derive(Debug)]
pub struct UiErr;

impl UiErr{
    pub fn new<T>(_: T)->Self{
        UiErr{}
    }
}

#[derive(Default, Copy, Clone)]
pub struct UiInput {
    pub read_addr: u16,
    pub write_addr: u16,
}

#[derive(Default, Copy, Clone)]
pub struct UiOutput {
    pub op1: OP1,
    pub op2: OP2,
    pub op1_en: bool,
    pub op2_en: bool,
    pub op1_arg: u16,
    pub op2_arg: u16,
    pub rev: bool,
    pub wr_prot: bool,
    pub fb_lr_swap: bool,
    pub fb_phase_flip: bool,
    pub fdbk_knob: u16,
    pub clk_knob: u16,
    pub mix_knob: u16,
}

#[derive(Default, Copy, Clone)]
pub enum OP1 {
    #[default]
    AND,
    MUL,
    OSC,
    RND,
}

#[derive(Default, Copy, Clone)]
pub enum OP2 {
    #[default]
    OR,
    XOR,
    MSK,
    SUB,
}

#[macro_export]
macro_rules! read_panel{
    ($out:ident, $i2c:ident, $expanders:ident, $adc:ident,
    $fdbk_knob: ident, $clk_knob: ident, $mix_knob: ident, $led_strip: ident) => {
        let mut op1: Option<ui::OP1> = None;
        let mut op2: Option<ui::OP2> = None;
        $expanders.read_opsel_en(&mut $i2c, &mut op1, &mut op2,
                                &mut $out.op1_en, &mut $out.op2_en).unwrap();
        // Only update op sel switch values if they are in a valid state
        if let Some(q) = op1{ $out.op1 = q; }
        if let Some(q) = op2{ $out.op2 = q; }
        $expanders.write_opsel_leds(&mut $i2c, &$out.op1, &$out.op2,
                                   &$out.op1_en, &$out.op2_en).unwrap();
        $expanders.read_args(&mut $i2c, &mut $out.op1_arg, &mut $out.op2_arg).unwrap();
        $expanders.read_settings(&mut $i2c, &mut $out.fb_phase_flip, &mut $out.fb_lr_swap,
                                &mut $out.wr_prot, &mut $out.rev).unwrap();
        if let Some(n) = $fdbk_knob.read(&mut $adc).unwrap(){
            $out.fdbk_knob = n;
            $led_strip.display_fdbk();
        }
        if let Some(n) = $clk_knob.read(&mut $adc).unwrap(){
            $out.clk_knob = n;
            $led_strip.display_clk();
        }
        if let Some(n) = $mix_knob.read(&mut $adc).unwrap(){
            $out.mix_knob = n;
            $led_strip.display_mix();
        }
    }
}