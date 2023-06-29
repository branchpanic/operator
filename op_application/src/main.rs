use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use iced::{Alignment, Application, Event, Length, Point, Rectangle, subscription, Theme, time, window};
use iced::{Command, Element, executor, Settings, Subscription};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::Event::{KeyPressed, KeyReleased};
use iced::keyboard::KeyCode;
use iced::widget::{button, slider, checkbox, column, container, pick_list, row, text};
use iced_native::widget::scrollable;

use op_engine::{Project, Session};

use crate::faust::{FaustDsp, FaustGenerator};
use crate::keyboard::Keyboard;

mod keyboard;
mod faust;
mod faust_engines;
mod view;

pub fn main() -> iced::Result {
    OpApplication::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

const BASE_SAMPLES_PER_PIXEL: i32 = 300;

struct OpApplication {
    session: Session,
    project_path: Option<PathBuf>,
    playing: bool,
    recording: bool,
    armed_track: usize,
    keyboard: Keyboard,
    held_keys: HashSet<KeyCode>,
    zoom: f32
}

#[derive(Debug, Clone)]
pub enum OpMessage {
    Play,
    Pause,
    Stop,
    PlaybackTick,
    SetRecording(bool),
    SetArmedTrack(usize),
    InputEvent(Event),
    Save,
    Load,
    Export,
    SetZoom(f32)
}

fn apply_default_generator(session: &mut Session) {
    let mut sine = faust_engines::Sine::new();
    let sample_rate = session.project.read().unwrap().sample_rate;
    sine.init(sample_rate as i32);
    let generator = FaustGenerator::new(Box::new(sine));
    session.set_generator(Box::new(generator));
}

impl Application for OpApplication {
    type Executor = executor::Default;
    type Message = OpMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut session = Session::new_empty().unwrap();
        apply_default_generator(&mut session);

        (
            Self {
                session,
                project_path: None,
                playing: false,
                recording: false,
                armed_track: 0,
                keyboard: Keyboard::new(),
                held_keys: HashSet::new(),
                zoom: 1.0,
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
                self.session.pause().unwrap();
                self.session.seek(0);
                self.recording = false;
                self.session.set_recording(false, self.armed_track);
                self.playing = false;
            }

            OpMessage::PlaybackTick => {
                // ...
            }

            OpMessage::SetRecording(recording) => {
                self.recording = recording;
                self.session.set_recording(true, self.armed_track);
            }

            OpMessage::SetArmedTrack(armed_track) => {
                if !self.playing {
                    self.armed_track = armed_track;
                    self.session.set_recording(self.recording, armed_track);
                }
            }

            OpMessage::InputEvent(event) => {
                match event {
                    Event::Keyboard(keyboard_event) => {
                        match keyboard_event {
                            KeyPressed { key_code: c, .. } => { self.held_keys.insert(c); }
                            KeyReleased { key_code: c, .. } => { self.held_keys.remove(&c); }
                            _ => {}
                        };

                        for msg in self.keyboard.update(&self.held_keys) {
                            self.session.handle(msg);
                        }
                    }
                    Event::Window(window::Event::CloseRequested) => { return window::close(); }
                    _ => {}
                };
            }

            OpMessage::SetZoom(zoom) => {
                self.zoom = zoom;
            }

            // TODO: Don't block UI to show the file dialog in save/load/export

            OpMessage::Save => {
                let project = self.session.project.read().unwrap();

                if let Some(path) = &self.project_path {
                    project.save(path).unwrap();
                } else {
                    let dialog = rfd::FileDialog::new();
                    let path = dialog.save_file();
                    if let Some(path) = path {
                        project.save(&path).unwrap();
                        self.project_path = Some(path);
                    }
                }
            }

            OpMessage::Load => {
                let dialog = rfd::FileDialog::new();
                let path = match dialog.pick_folder() {
                    None => return Command::none(),
                    Some(path) => path
                };

                let project = Project::load(&path).unwrap();
                let mut session = Session::new_with_project(project).unwrap();
                apply_default_generator(&mut session);

                self.project_path = Some(path);
                self.session = session;
                self.playing = false;
                self.recording = false;
                self.armed_track = 0;
            }

            OpMessage::Export => {
                let dialog = rfd::FileDialog::new().add_filter("WAV", &["wav"]);

                let path = match dialog.save_file() {
                    None => return Command::none(),
                    Some(path) => path
                };

                let project = self.session.project.read().unwrap();
                project.export_wav(&path).unwrap();
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let project = self.session.project.read().unwrap();
        let tracks: Vec<usize> = (0..project.timeline.tracks.len()).collect();

        let transport_controls = row![
            if !self.playing {
                button("Play").on_press(OpMessage::Play)
            } else {
                button("Pause").on_press(OpMessage::Pause)
            },
            button("Stop").on_press(OpMessage::Stop),
            checkbox("Record", self.recording, OpMessage::SetRecording),
            pick_list(tracks, Some(self.armed_track), OpMessage::SetArmedTrack)
        ].spacing(4);

        let status_display = row![
            text(format!("{}", self.session.time()))
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center),
        ];

        let project_controls = container(row![
            button("Load").on_press(OpMessage::Load),
            button("Save").on_press(OpMessage::Save),
            button("Export").on_press(OpMessage::Export),
        ].spacing(4)).align_x(Horizontal::Right);

        let top_bar = container(row![
            transport_controls.align_items(Alignment::Center).width(Length::FillPortion(1)),
            status_display.align_items(Alignment::Center).width(Length::FillPortion(2)),
            project_controls.width(Length::FillPortion(1)),
        ])
            .padding(8)
            .width(Length::Fill);

        let timeline = container(
            scrollable(column(
            project.timeline.tracks.iter().enumerate()
                .map(|(i, track)| {
                    row![
                        text(format!("{}", i)).height(Length::Fill).vertical_alignment(Vertical::Center),
                        container(row(track.iter_clips().map(|clip_inst| {
                            view::timeline_clip::clip_view(
                                clip_inst.clip.clone(),
                                (self.zoom * BASE_SAMPLES_PER_PIXEL as f32) as usize
                            )
                        }).collect()))
                    ].padding(20.0).spacing(15.0).height(128.0).into()
                })
                .collect())))
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill);

        column![
            top_bar,
            timeline,
            slider(0.05..=5.0, self.zoom, OpMessage::SetZoom).step(0.1)
        ].into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            if self.playing {
                time::every(Duration::from_millis(10)).map(|_| OpMessage::PlaybackTick)
            } else {
                Subscription::none()
            },
            subscription::events().map(OpMessage::InputEvent),
        ])
    }
}
