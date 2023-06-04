mod keyboard;
mod faust;
mod sine;

use std::time::Duration;
use iced::{Application, theme, time};
use iced::{Command, Element, executor, Sandbox, Settings, Subscription};
use iced::widget::{button, checkbox, row, text};
use op_engine::{Clip, Session};
use crate::sine::Sine;
use crate::faust::{FaustDsp, FaustGenerator};
// use crate::keyboard::Keyboard;

pub fn main() -> iced::Result {
    OpApplication::run(Settings::default())
}

struct OpApplication {
    session: Session,
    playing: bool,
    recording: bool,
    record_track: usize,
    time: usize,
}

#[derive(Debug, Clone)]
enum OpMessage {
    Play,
    Pause,
    Stop,
    Tick,
    SetRecording(bool),
}

impl Application for OpApplication {
    type Executor = executor::Default;
    type Message = OpMessage;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                session: Session::empty_with_defaults().unwrap(),
                playing: false,
                recording: false,
                record_track: 0,
                time: 0,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "op_application".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            OpMessage::Play => {
                if !self.playing {
                    self.session.play().unwrap();
                    self.playing = true;
                }
            }

            OpMessage::Pause => {
                if self.playing {
                    self.session.pause().unwrap();
                    self.playing = false;
                }
            }

            OpMessage::Stop => {
                if self.playing {
                    self.session.pause().unwrap();
                    self.session.set_recording(false, self.record_track);
                }

                self.playing = false;
                self.session.seek(0);
            }

            OpMessage::Tick => {
                self.time = self.session.time();
            }

            OpMessage::SetRecording(recording) => {
                self.recording = recording;
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        row![
            if !self.playing {
                button("Play").on_press(OpMessage::Play)
            } else {
                button("Pause").on_press(OpMessage::Pause)
            },
            button("Stop").on_press(OpMessage::Stop),
            checkbox("Record", self.recording, OpMessage::SetRecording),
            text(self.time),
        ].spacing(10).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.playing {
            time::every(Duration::from_millis(10)).map(|_| OpMessage::Tick)
        } else {
            Subscription::none()
        }
    }
}

// fn main() -> eframe::Result<()> {
//     let options = eframe::NativeOptions {
//         initial_window_size: Some(egui::Vec2::new(800.0, 600.0)),
//         ..Default::default()
//     };
//
//     eframe::run_native(
//         "op_application",
//         options,
//         Box::new(|_| Box::new(Application::new().unwrap())),
//     )
// }
//
// struct Application {
//     session: Session,
//     keyboard: Keyboard,
//     load_path: String,
//     load_track: usize,
//     load_time_sec: f32,
//     recording: bool,
//     record_track: usize,
//     playing: bool,
// }
//
// impl Application {
//     fn new() -> anyhow::Result<Self> {
//         let session = Session::empty_with_defaults()?;
//
//         {
//             let mut project = session.project.lock().unwrap();
//             let mut sine = Sine::new();
//             sine.init(project.sample_rate as i32);
//             let generator = FaustGenerator::new(Box::new(sine));
//             project.generator = Box::new(generator);
//         }
//
//         Ok(Self {
//             session,
//             load_path: "".to_string(),
//             load_track: 0,
//             load_time_sec: 0.0,
//             recording: false,
//             record_track: 0,
//             playing: false,
//             keyboard: Keyboard::new(),
//         })
//     }
// }
//
// impl eframe::App for Application {
//     fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
//         ctx.input(|i| {
//             let result = self.keyboard.update(&i.keys_down);
//             result.into_iter().for_each(|m| self.session.handle(m));
//         });
//
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.horizontal(|ui| {
//                 if ui.button("Load").clicked() {
//                     if let Some(path) = rfd::FileDialog::new().pick_folder() {
//                         let mut project = self.session.project.lock().unwrap();
//                         project.load_overwrite(&path.display().to_string()).unwrap();
//                     };
//                 }
//
//                 if ui.button("Save").clicked() {
//                     if let Some(path) = rfd::FileDialog::new().save_file() {
//                         let project = self.session.project.lock().unwrap();
//                         project.save(&path.display().to_string()).unwrap();
//                     };
//                 }
//
//                 if ui.button("Export").clicked() {
//                     let dialog = rfd::FileDialog::new()
//                         .add_filter("WAV audio", &["wav"]);
//
//                     if let Some(path) = dialog.save_file() {
//                         let project = self.session.project.lock().unwrap();
//                         project.export_wav(&path.display().to_string()).unwrap();
//                     };
//                 }
//             });
//
//             ui.horizontal(|ui| {
//                 if ui.button("Play").clicked() {
//                     self.session.play().unwrap();
//                     self.playing = true;
//                 }
//
//                 if ui.button("Pause").clicked() {
//                     self.session.pause().unwrap();
//                     self.playing = false;
//                 }
//
//                 if ui.button("Stop").clicked() {
//                     self.session.pause().unwrap();
//                     self.session.seek(0);
//
//                     self.recording = false;
//                     self.session.set_recording(false, self.record_track);
//                     self.playing = false;
//                 }
//
//                 let record_toggle = ui.toggle_value(&mut self.recording, "Record");
//
//                 ui.label("Track:");
//                 egui::DragValue::new(&mut self.record_track)
//                     .clamp_range(RangeInclusive::new(0, 3))
//                     .ui(ui);
//
//                 if record_toggle.changed() {
//                     self.session.set_recording(self.recording, self.record_track);
//                 }
//
//                 ui.label("Sample:");
//                 ui.label(format!("{}", self.session.time()));
//             });
//
//             ui.add_space(8.0);
//
//             ui.label("Add clip");
//             ui.horizontal(|ui| {
//                 if ui.button("Open").clicked() {
//                     let dialog = rfd::FileDialog::new()
//                         .add_filter("WAV audio", &["wav"]);
//
//                     if let Some(path) = dialog.pick_file() {
//                         self.load_path = path.display().to_string();
//                     }
//                 }
//
//                 ui.text_edit_singleline(&mut self.load_path);
//
//                 ui.label("Track:");
//                 egui::DragValue::new(&mut self.load_track)
//                     .clamp_range(RangeInclusive::new(0, 3))
//                     .ui(ui);
//
//                 ui.label("Start:");
//                 egui::DragValue::new(&mut self.load_time_sec)
//                     .clamp_range(RangeInclusive::new(0, 100))
//                     .ui(ui);
//
//                 if ui.button("Add").clicked() {
//                     let mut project = self.session.project.lock().unwrap();
//                     let sec = project.sec_to_samples(self.load_time_sec);
//                     match Clip::load_wav(&project, &self.load_path) {
//                         Ok(clip) => {
//                             project.timeline.tracks[self.load_track].add_clip(sec, clip);
//                         }
//                         Err(e) => {
//                             eprintln!("failed to load clip: {}", e);
//                         }
//                     }
//                 }
//             });
//
//             ui.add_space(8.0);
//
//             egui::Grid::new("clips")
//                 .min_col_width(25.0)
//                 .striped(true)
//                 .show(ui, |ui| {
//                     ui.label("Track");
//                     ui.label("Start");
//                     ui.label("Length");
//                     ui.end_row();
//
//                     let project = self.session.project.lock().unwrap();
//                     for (track, inst) in project.timeline.iter_clips() {
//                         ui.label(format!("{}", track));
//                         ui.label(format!("{}", inst.start()));
//                         ui.label(format!("{}", inst.len()));
//                     }
//                 });
//         });
//
//         if self.playing {
//             ctx.request_repaint();
//         }
//     }
// }
