use std::iter;

use iced::{Color, Element, Length, Point, Rectangle, Theme};
use iced::alignment::Vertical;
use iced::mouse::Interaction;
use iced::widget::Canvas;
use iced::widget::canvas::{Cursor, Event, Fill, Frame, Geometry, LineCap, LineJoin, Path, Program, Stroke, Style};
use iced_native::event::Status;
use iced_native::row;
use iced_native::widget::{column, container, text};

use op_engine::clip_database::{ClipDatabase, ClipId};
use op_engine::track::ClipInstance;

use crate::OpMessage;

const BASE_SAMPLES_PER_PIXEL: f32 = 300.0;
const BASE_RULER_SPACING_SAMPLES: f32 = 22050.0;

struct ClipLayout {
    clip_id: ClipId,
    waveform: Vec<f32>,

    x: f32,
    width: f32,
}

impl ClipLayout {
    fn new(clip_instance: &ClipInstance, clip_db: &ClipDatabase, zoom: f32, start_time: op_engine::Time) -> Self {
        let clip = clip_db.get(clip_instance.clip_id).expect("TODO: Missing clip UI");

        Self {
            clip_id: clip_instance.clip_id,
            waveform: clip.data.chunks((zoom * BASE_SAMPLES_PER_PIXEL) as usize)
                .map(|chunk| {
                    chunk.iter().map(|s| s.abs()).sum::<f32>() / (chunk.len() as f32)
                })
                .collect(),

            x: zoom * BASE_SAMPLES_PER_PIXEL * (clip_instance.time - start_time) as f32,
            width: clip.len() as f32 / (zoom * BASE_SAMPLES_PER_PIXEL),
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

    fn waveform_y(y: &f32, height: f32) -> f32 {
        1.0 * (1.0 - y.abs()) * (height - 12.0)
    }

    pub fn draw(&self, bounds: &Rectangle, hovered: bool) -> impl Iterator<Item=Geometry> {
        let mut frame = Frame::new(bounds.size());

        if self.waveform.len() > 0 {
            let mut point = Point::new(self.x, Self::waveform_y(&self.waveform[0], bounds.height));

            let path = Path::new(|builder| {
                builder.move_to(point);

                for y in self.waveform.iter().skip(1) {
                    point.x += 1.0;
                    point.y = Self::waveform_y(y, bounds.height);
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
    pub fn new(track: &op_engine::Track, clip_db: &ClipDatabase, zoom: f32, current_time: op_engine::Time) -> Self {
        Self {
            zoom,
            current_time,
            start_time: 0,
            clip_layouts: track.iter_clips()
                .map(|c| { ClipLayout::new(c, clip_db, zoom, 0) })
                .collect(),
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

    fn draw_ruler(&self, bounds: &Rectangle) -> impl Iterator<Item=Geometry> {
        let marks = 20;

        let first_mark_number = (self.start_time / BASE_RULER_SPACING_SAMPLES as usize);
        let first_mark_time = first_mark_number * BASE_RULER_SPACING_SAMPLES as usize;

        let path = Path::new(|builder| {
            for i in 0..marks {
                let time = first_mark_time + i * BASE_RULER_SPACING_SAMPLES as usize;
                let x = time as f32 / (self.zoom * BASE_SAMPLES_PER_PIXEL);
                builder.move_to(Point::new(x, 0.0));
                builder.line_to(Point::new(x, bounds.height));
            }
        });

        let mut frame = Frame::new(bounds.size());
        frame.stroke(&path, Stroke::default()
            .with_width(1.0)
            .with_color(Color::from_rgb(0.22, 0.22, 0.22)));

        let background = frame.into_geometry();
        iter::once(background)
    }

    fn draw_playhead(&self, bounds: &Rectangle) -> impl Iterator<Item=Geometry> {
        let playhead_relative_x = self.current_time - self.start_time;
        let x = playhead_relative_x as f32 / (self.zoom * BASE_SAMPLES_PER_PIXEL);

        let line = Path::line(
            Point::new(x, 0.0),
            Point::new(x, bounds.height),
        );

        let mut frame = Frame::new(bounds.size());
        frame.stroke(&line, Stroke::default()
            .with_width(2.0)
            .with_color(Color::WHITE));

        let handle = Path::new(|builder| {
            builder.move_to(Point::new(x - 5.0, 0.0));
            builder.line_to(Point::new(x + 5.0, 0.0));
            builder.line_to(Point::new(x, 10.0));
        });

        frame.fill(&handle, Fill {
            style: Style::Solid(Color::WHITE),
            ..Default::default()
        });

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
            .chain(self.draw_ruler(&bounds))
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

fn track_view(number: usize, track: &op_engine::Track, clip_db: &ClipDatabase, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
    let prog = TrackProgram::new(track, clip_db, zoom, current_time);
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

pub fn timeline_view(timeline: &op_engine::Timeline, clip_db: &ClipDatabase, zoom: f32, current_time: usize) -> Element<'static, OpMessage> {
    container(
        column(timeline.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                track_view(i, track, clip_db, zoom, current_time)
            })
            .collect())
    )
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}