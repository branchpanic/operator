use midly::MidiMessage;

use op_engine::generator::Generator;

#[derive(Copy, Clone)]
pub struct ParamIndex(pub i32);

pub type F32 = f32;

pub trait FaustDsp: Send {
    type T;

    fn new() -> Self where Self: Sized;
    fn metadata(&self, m: &mut dyn Meta);
    fn get_sample_rate(&self) -> i32;
    fn get_num_inputs(&self) -> i32;
    fn get_num_outputs(&self) -> i32;
    fn class_init(sample_rate: i32) where Self: Sized;
    fn instance_reset_params(&mut self);
    fn instance_clear(&mut self);
    fn instance_constants(&mut self, sample_rate: i32);
    fn instance_init(&mut self, sample_rate: i32);
    fn init(&mut self, sample_rate: i32);
    fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>);
    fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) where Self: Sized;
    fn get_param(&self, param: ParamIndex) -> Option<Self::T>;
    fn set_param(&mut self, param: ParamIndex, value: Self::T);
    fn compute(&mut self, count: i32, inputs: &[&[Self::T]], outputs: &mut [&mut [Self::T]]);
}

pub trait Meta {
    // -- metadata declarations
    fn declare(&mut self, key: &str, value: &str);
}

pub trait UI<T> {
    // -- widget's layouts
    fn open_tab_box(&mut self, label: &str);
    fn open_horizontal_box(&mut self, label: &str);
    fn open_vertical_box(&mut self, label: &str);
    fn close_box(&mut self);

    // -- active widgets
    fn add_button(&mut self, label: &str, param: ParamIndex);
    fn add_check_button(&mut self, label: &str, param: ParamIndex);
    fn add_vertical_slider(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);
    fn add_horizontal_slider(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);
    fn add_num_entry(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);

    // -- passive widgets
    fn add_horizontal_bargraph(&mut self, label: &str, param: ParamIndex, min: T, max: T);
    fn add_vertical_bargraph(&mut self, label: &str, param: ParamIndex, min: T, max: T);

    // -- metadata declarations
    fn declare(&mut self, param: Option<ParamIndex>, key: &str, value: &str);
}

pub struct FaustGenerator {
    faust_dsp: Box<dyn FaustDsp<T=F32>>,
    last_note: u8,
}

impl FaustGenerator {
    pub fn new(faust_dsp: Box<dyn FaustDsp<T=F32>>) -> Self {
        Self {
            faust_dsp,
            last_note: 0,
        }
    }
}

fn midi_note_to_hz(note: u8) -> f64 {
    440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
}

impl Generator for FaustGenerator {
    fn next(&mut self) -> f32 {
        let input = [0.0; 1];
        let mut output = [0.0; 1];
        self.faust_dsp.compute(1, &[&input], &mut [&mut output]);
        output[0]
    }

    fn handle(&mut self, msg: MidiMessage) {
        match msg {
            MidiMessage::NoteOn { key, .. } => {
                self.last_note = key.as_int();
                self.faust_dsp.set_param(ParamIndex(0), midi_note_to_hz(key.into()) as f32); // freq
                self.faust_dsp.set_param(ParamIndex(1), 1.0); // gate
            }
            MidiMessage::NoteOff { key, .. } => {
                if self.last_note == key.as_int() {
                    self.faust_dsp.set_param(ParamIndex(1), 0.0);
                }
            }
            _ => ()
        }
    }
}