mod keyboard;

use std::ops::RangeInclusive;

use eframe::Frame;
use egui::{Context, Widget};

use op_engine::{Clip, Session};
use crate::keyboard::Keyboard;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "op_application",
        options,
        Box::new(|_| Box::new(Application::new().unwrap())),
    )
}

struct Application {
    session: Session,
    load_path: String,
    load_track: usize,
    load_time_sec: f32,
    recording: bool,
    playing: bool,
    keyboard: Keyboard,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            session: Session::empty_with_defaults()?,
            load_path: "".to_string(),
            load_track: 0,
            load_time_sec: 0.0,
            recording: false,
            playing: false,
            keyboard: Keyboard::new(),
        })
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| {
            let result = self.keyboard.update(&i.keys_down);
            result.into_iter().for_each(|m| self.session.handle(m));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let mut project = self.session.project.lock().unwrap();
                        project.load_overwrite(&path.display().to_string()).unwrap();
                    };
                }

                if ui.button("Save").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        let project = self.session.project.lock().unwrap();
                        project.save(&path.display().to_string()).unwrap();
                    };
                }

                if ui.button("Export").clicked() {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("WAV audio", &["wav"]);

                    if let Some(path) = dialog.save_file() {
                        let project = self.session.project.lock().unwrap();
                        project.export_wav(&path.display().to_string()).unwrap();
                    };
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    self.session.play().unwrap();
                    self.playing = true;
                }

                if ui.button("Pause").clicked() {
                    self.session.pause().unwrap();
                    self.playing = false;
                }

                if ui.button("Stop").clicked() {
                    self.session.pause().unwrap();
                    self.session.seek(0);

                    self.recording = false;
                    self.session.set_recording(false);
                    self.playing = false;
                }

                if ui.toggle_value(&mut self.recording, "Record").changed() {
                    self.session.set_recording(self.recording);
                }

                ui.label(format!("{}", self.session.time()));
            });

            ui.add_space(8.0);

            ui.heading("Add clip");
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("WAV audio", &["wav"]);

                    if let Some(path) = dialog.pick_file() {
                        self.load_path = path.display().to_string();
                    }
                }

                ui.text_edit_singleline(&mut self.load_path);

                ui.label("Track:");
                egui::DragValue::new(&mut self.load_track)
                    .clamp_range(RangeInclusive::new(0, 4))
                    .ui(ui);

                ui.label("Start:");
                egui::DragValue::new(&mut self.load_time_sec)
                    .clamp_range(RangeInclusive::new(0, 100))
                    .ui(ui);

                if ui.button("Add").clicked() {
                    let mut project = self.session.project.lock().unwrap();
                    let sec = project.sec_to_samples(self.load_time_sec);
                    match Clip::load_wav(&project, &self.load_path) {
                        Ok(clip) => {
                            project.timeline.tracks[self.load_track].add_clip(sec, clip);
                        }
                        Err(e) => {
                            eprintln!("failed to load clip: {}", e);
                        }
                    }
                }
            });

            ui.add_space(8.0);

            egui::Grid::new("clips")
                .min_col_width(25.0)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Track");
                    ui.label("Start");
                    ui.label("Length");
                    ui.end_row();

                    let project = self.session.project.lock().unwrap();
                    for (track, inst) in project.timeline.iter_clips() {
                        ui.label(format!("{}", track));
                        ui.label(format!("{}", inst.start()));
                        ui.label(format!("{}", inst.len()));
                        ui.end_row();
                    }
                });
        });

        if self.playing {
            ctx.request_repaint();
        }
    }
}
