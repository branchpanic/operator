use std::ops::RangeInclusive;
use eframe::Frame;
use egui::{Context, Widget};
use op_engine::{Clip, Session};

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
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            session: Session::empty_with_defaults()?,
            load_path: "".to_string(),
            load_track: 0,
            load_time_sec: 0.0,
        })
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
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
                        project.export(&path.display().to_string()).unwrap();
                    };
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    self.session.play().unwrap();
                }

                if ui.button("Pause").clicked() {
                    self.session.pause().unwrap();
                }

                if ui.button("Stop").clicked() {
                    self.session.pause().unwrap();
                    self.session.seek(0);
                }
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
                    match Clip::from_file(&self.load_path) {
                        Ok(clip) => {
                            project.add_clip(self.load_track, sec, clip);
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
                    ui.end_row();

                    let project = self.session.project.lock().unwrap();
                    for clip in project.iter_clips() {
                        ui.label(format!("{}", clip.track));
                        ui.label(format!("{}", clip.start));
                        ui.end_row();
                    }
                });
        });
    }
}
