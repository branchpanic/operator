use std::iter;

use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};
use iced::alignment::Vertical;
use iced::mouse::Interaction;
use iced::widget::Canvas;
use iced::widget::canvas::{Cursor, Event, Frame, Geometry, LineCap, LineJoin, Path, Program, Stroke};
use iced_native::event::Status;
use iced_native::row;
use iced_native::widget::{column, container, text};

use op_engine::track::ClipInstance;

use crate::OpMessage;

const BASE_SAMPLES_PER_PIXEL: f32 = 300.0;

struct ClipLayout {
    clip_instance: ClipInstance,
    zoom: f32,
    samples_per_pixel: usize,
    x: f32,
    width: f32,
}

impl ClipLayout {
    fn new(clip: ClipInstance, zoom: f32, start_time: op_engine::Time) -> Self {
        Self {
            zoom,
            samples_per_pixel: (BASE_SAMPLES_PER_PIXEL * zoom) as usize,
            x: zoom * BASE_SAMPLES_PER_PIXEL * (clip.time - start_time) as f32,
            width: clip.len() as f32 / (zoom * BASE_SAMPLES_PER_PIXEL),
            clip_instance: clip,
        }
    }

    fn clip_bounds(&self, parent_bounds: &Rectangle) -> Rectangle {
        Rectangle {
            x: parent_bounds.x + self.x,
            y: parent_bounds.y,
            width: self.width,
            height: parent_bounds.height,
        }
    }

    pub fn draw(&self, bounds: &Rectangle, hovered: bool) -> impl Iterator<Item=Geometry> {
        let mut frame = Frame::new(bounds.size());

        if self.clip_instance.clip.data.len() > 0 {
            let get_y = |sample: f32| {
                1.0 * (1.0 - sample.abs()) * (bounds.height - 12.0)
            };

            let mut point = Point::ORIGIN;
            let path = Path::new(|builder| {
                builder.move_to(Point::new(self.x, get_y(self.clip_instance.clip.data[0])));

                for i in 0..self.clip_instance.clip.data.len() / self.samples_per_pixel {
                    let mut sample = 0.0;

                    for j in 0..self.samples_per_pixel {
                        sample += self.clip_instance.clip.data[i * self.samples_per_pixel + j].abs();
                    }

                    sample /= self.samples_per_pixel as f32;
                    point.x = self.x + i as f32;
                    point.y = get_y(sample);
                    builder.line_to(point);
                }

                builder.circle(Point::new(point.x + 5.0, point.y), 5.0);
            });

            frame.stroke(&path, Stroke::default()
                .with_width(if hovered { 4.0 } else { 2.0 })
                .with_color(Color::WHITE)
                .with_line_cap(LineCap::Square)
                .with_line_join(LineJoin::Bevel));
        }

        iter::once(frame.into_geometry())
    }
}

pub struct TrackProgram {
    zoom: f32,
    start_time: op_engine::Time,
    current_time: op_engine::Time,
    clip_layouts: Vec<ClipLayout>,
}

#[derive(Default)]
pub struct TrackProgramState {
    hovered_clip: Option<usize>,
}

impl TrackProgram {
    pub fn new(track: &op_engine::Track, zoom: f32, current_time: op_engine::Time) -> Self {
        Self {
            zoom,
            current_time,
            start_time: 0,

            // TODO: Borrow
            clip_layouts: track.iter_clips().map(|c| { ClipLayout::new(c.clone(), zoom, 0) }).collect(),
        }
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
        let playhead_relative_x = self.current_time - self.start_time;
        let x = playhead_relative_x as f32 / (self.zoom * BASE_SAMPLES_PER_PIXEL);

        let path = Path::line(
            Point::new(x, 0.0),
            Point::new(x, bounds.height),
        );

        let mut frame = Frame::new(bounds.size());
        frame.stroke(&path, Stroke::default()
            .with_width(2.0)
            .with_color(Color::WHITE));
        let background = frame.into_geometry();
        iter::once(background)
    }
}

impl Program<OpMessage> for TrackProgram {
    type State = TrackProgramState;

    fn update(&self, state: &mut Self::State, _event: Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<OpMessage>) {
        state.hovered_clip = self.clip_layouts.iter()
            .enumerate()
            .find(|(_, c)| {
                let clip_bounds = c.clip_bounds(&bounds);
                cursor.is_over(&clip_bounds)
            })
            .map(|(i, _)| i);

        (Status::Ignored, None)
    }

    fn draw(&self, state: &Self::State, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        self.draw_baseline(&bounds)
            .chain(self.draw_playhead(&bounds))
            .chain(self.clip_layouts.iter().enumerate().flat_map(|(i, c)| { c.draw(&bounds, Some(i) == state.hovered_clip) }))
            .collect()
    }

    fn mouse_interaction(&self, state: &Self::State, _bounds: Rectangle, _cursor: Cursor) -> Interaction {
        match state.hovered_clip {
            Some(_) => Interaction::Grab,
            _ => Interaction::default(),
        }
    }
}

fn track_view(number: usize, track: &op_engine::Track, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
    let prog = TrackProgram::new(track, zoom, current_time);
    let clip_area = Canvas::new(prog).width(Length::Fill);

    let track_header = text(format!("{}", number))
        .height(Length::Fill)
        .vertical_alignment(Vertical::Center);

    row![track_header, clip_area]
        .padding(20.0)
        .spacing(15.0)
        .height(Length::Fill)
        .into()
}

pub fn timeline_view(timeline: &op_engine::Timeline, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
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