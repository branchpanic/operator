use std::iter;

use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};
use iced::alignment::Vertical;
use iced::widget::{canvas, Canvas};
use iced::widget::canvas::{Cursor, Frame, Geometry, Path, Stroke};
use iced_native::row;
use iced_native::widget::{column, container, text};

use op_engine::{Timeline, Track};

use crate::clip_view::ClipProgram;
use crate::OpMessage;

const BASE_SAMPLES_PER_PIXEL: f32 = 300.0;

pub struct TrackProgram {
    track: Track,
    zoom: f32,
    start: op_engine::Time,
    current_time: op_engine::Time,
}

impl TrackProgram {
    fn samples_to_length(&self, samples: usize) -> f32 {
        self.zoom * BASE_SAMPLES_PER_PIXEL * samples as f32
    }

    fn draw_baseline(&self, bounds: &Rectangle) -> impl Iterator<Item=Geometry> {
        let path = Path::line(
            Point::new(0.0, bounds.height - 12.0),
            Point::new(bounds.width, bounds.height - 12.0),
        );

        let mut frame = Frame::new(bounds.size());
        frame.stroke(&path, Stroke::default().with_width(2.0).with_color(Color::from_rgb(0.25, 0.25, 0.25)));
        let background = frame.into_geometry();
        iter::once(background)
    }

    fn draw_playhead(&self, bounds: &Rectangle) -> impl Iterator<Item=Geometry> {
        let playhead_relative_x = self.current_time - self.start;
        let x = playhead_relative_x as f32 / (self.zoom * BASE_SAMPLES_PER_PIXEL);

        let path = Path::line(
            Point::new(x, 0.0),
            Point::new(x, bounds.height),
        );

        let mut frame = Frame::new(bounds.size());
        frame.stroke(&path, Stroke::default().with_width(2.0).with_color(Color::WHITE));
        let background = frame.into_geometry();
        iter::once(background)
    }
}

impl canvas::Program<OpMessage> for TrackProgram {
    type State = ();

    fn draw(&self, state: &Self::State, theme: &Theme, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let clips = self.track.iter_clips()
            .flat_map(|clip_inst| {
                let prog = ClipProgram {
                    clip: clip_inst.clip.clone(),
                    samples_per_pixel: (self.zoom * BASE_SAMPLES_PER_PIXEL) as usize,
                };

                let x = (clip_inst.time - self.start) as f32 / (self.zoom * BASE_SAMPLES_PER_PIXEL);
                let width = self.samples_to_length(clip_inst.clip.len());

                let clip_bounds = Rectangle::new(
                    Point::new(x, 0.0),
                    Size::new(width, bounds.height),
                );

                prog.draw(state, theme, clip_bounds, cursor)
            });

        self.draw_baseline(&bounds)
            .chain(self.draw_playhead(&bounds))
            .chain(clips)
            .collect()
    }
}

fn track_view(number: usize, track: &Track, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
    let clip_area = Canvas::new(TrackProgram {
        // TODO: Borrow
        track: track.clone(),
        zoom,
        start: 0,
        current_time,
    }).width(Length::Fill);

    let track_header = text(format!("{}", number))
        .height(Length::Fill)
        .vertical_alignment(Vertical::Center);

    row![track_header, clip_area]
        .padding(20.0)
        .spacing(15.0)
        .height(Length::Fill)
        .into()
}

pub fn timeline_view(timeline: &Timeline, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
    container(
        column(timeline.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                track_view(i, track, zoom, current_time)
            })
            .collect()))
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}