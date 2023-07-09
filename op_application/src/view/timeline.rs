use std::iter;

use iced::{Color, Element, Length, mouse, Point, Rectangle, Theme};
use iced::alignment::Vertical;
use iced::mouse::Interaction;
use iced::widget::Canvas;
use iced::widget::canvas::{Cursor, Event, Fill, Frame, Geometry, LineCap, LineJoin, Path, Program, Stroke, Style};
use iced_native::event::Status;
use iced_native::row;
use iced_native::widget::{column, container, text};

use op_engine::clip_database::{ClipDatabase, ClipId};
use op_engine::track::ClipInstance;

const BASE_SAMPLES_PER_PIXEL: f32 = 300.0;
const BASE_RULER_SPACING_SAMPLES: f32 = 22050.0;

fn samples_to_pixels(samples: i32, zoom: f32) -> f32 {
    let samples_per_pixel = BASE_SAMPLES_PER_PIXEL * zoom;
    samples as f32 / samples_per_pixel
}

fn pixels_to_samples(pixels: f32, zoom: f32) -> i32 {
    let samples_per_pixel = BASE_SAMPLES_PER_PIXEL * zoom;
    (pixels * samples_per_pixel) as i32
}

struct ClipLayout {
    clip_id: ClipId,
    waveform: Vec<f32>,

    x: f32,
    width: f32,
    zoom: f32,
}

impl ClipLayout {
    fn new(clip_instance: &ClipInstance, clip_db: &ClipDatabase, zoom: f32, start_time: op_engine::Time) -> Self {
        let clip = clip_db.get(clip_instance.clip_id).expect("TODO: Missing clip UI");

        Self {
            clip_id: clip_instance.clip_id,
            waveform: clip.data.chunks(pixels_to_samples(1.0, zoom) as usize)
                .map(|chunk| {
                    chunk.iter().map(|s| s.abs()).sum::<f32>() / (chunk.len() as f32)
                })
                .collect(),

            x: samples_to_pixels((clip_instance.time - start_time) as i32, zoom),
            width: samples_to_pixels(clip.len() as i32, zoom),
            zoom,
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

    pub fn draw(&self, bounds: &Rectangle, hovered: bool, offset: i32) -> impl Iterator<Item=Geometry> {
        let mut frame = Frame::new(bounds.size());
        if self.waveform.len() > 0 {
            let mut point = Point::new(self.x + samples_to_pixels(offset, self.zoom), Self::waveform_y(&self.waveform[0], bounds.height));

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
    hovered_clip: Option<ClipId>,
    dragging_clip: Option<ClipId>,
    drag_origin: i32,
    drag_current: i32,
}

#[derive(Debug, Clone)]
pub enum TrackMessage {
    MoveClip { clip_id: ClipId, delta_samples: i32 },
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

        let first_mark_number = self.start_time / BASE_RULER_SPACING_SAMPLES as usize;
        let first_mark_time = first_mark_number * BASE_RULER_SPACING_SAMPLES as usize;

        let path = Path::new(|builder| {
            for i in 0..marks {
                let time = first_mark_time + i * BASE_RULER_SPACING_SAMPLES as usize;
                let x = samples_to_pixels(time as i32, self.zoom);
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
        let x = samples_to_pixels(playhead_relative_x as i32, self.zoom);

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

impl Program<TrackMessage> for TrackProgram {
    type State = TrackProgramState;

    fn update(&self, state: &mut Self::State, event: Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<TrackMessage>) {
        state.hovered_clip = self.clip_layouts.iter()
            .find(|c| {
                let clip_bounds = c.clip_bounds(&bounds);
                cursor.is_over(&clip_bounds)
            })
            .map(|c| c.clip_id);

        if let Event::Mouse(mouse::Event::CursorMoved { position, .. }) = event {
            state.drag_current = pixels_to_samples(position.x, self.zoom);
        }

        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
            if let Some(clip_id) = state.dragging_clip {
                println!("Released clip {:?}, change of {:?} samples", clip_id, state.drag_current - state.drag_origin);
                state.dragging_clip = None;
                return (Status::Captured, Some(TrackMessage::MoveClip { clip_id, delta_samples: state.drag_current - state.drag_origin }));
            }
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(clip_id) = state.hovered_clip {
                println!("Pressed clip {:?}", clip_id);
                state.dragging_clip = Some(clip_id);

                if let Some(cursor_pos) = cursor.position() {
                    state.drag_origin = pixels_to_samples(cursor_pos.x, self.zoom);
                }

                return (Status::Captured, None);
            }
        }

        (Status::Ignored, None)
    }

    fn draw(&self, state: &Self::State, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        self.draw_baseline(&bounds)
            .chain(self.draw_ruler(&bounds))
            .chain(self.draw_playhead(&bounds))
            .chain(self.clip_layouts.iter().flat_map(|c| {
                let is_dragging = Some(c.clip_id) == state.dragging_clip;
                let is_highlighted = is_dragging || (state.dragging_clip.is_none() && Some(c.clip_id) == state.hovered_clip);
                let offset = if is_dragging { state.drag_current - state.drag_origin } else { 0 };

                c.draw(&bounds, is_highlighted, offset)
            }))
            .collect()
    }

    fn mouse_interaction(&self, state: &Self::State, _bounds: Rectangle, _cursor: Cursor) -> Interaction {
        if state.dragging_clip.is_some() {
            return Interaction::Grabbing;
        }

        match state.hovered_clip {
            Some(_) => Interaction::Grab,
            _ => Interaction::default(),
        }
    }
}

fn track_view(number: usize, track: &op_engine::Track, clip_db: &ClipDatabase, zoom: f32, current_time: usize) -> Element<'static, TrackMessage> {
    let program = TrackProgram::new(track, clip_db, zoom, current_time);
    let clip_area = Canvas::new(program).width(Length::Fill);

    let track_header = text(format!("{}", number))
        .height(Length::Fill)
        .vertical_alignment(Vertical::Center);

    row![track_header, clip_area]
        .padding(20.0)
        .spacing(15.0)
        .height(Length::Fill)
        .into()
}

#[derive(Debug, Clone)]
pub enum TimelineMessage {
    Track(usize, TrackMessage),
}

pub fn timeline_view(timeline: &op_engine::Timeline, clip_db: &ClipDatabase, zoom: f32, current_time: usize) -> Element<'static, TimelineMessage> {
    container(
        column(timeline.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                track_view(i, track, clip_db, zoom, current_time).map(move |m| TimelineMessage::Track(i, m))
            })
            .collect())
    )
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn track_update(track: &mut op_engine::Track, message: TrackMessage) {
    match message {
        TrackMessage::MoveClip { clip_id, delta_samples, .. } => {
            if let Some(mut instance) = track.get_clip_mut(clip_id) {
                instance.time = (instance.time as i32 + delta_samples) as usize;
            }
        }
    }
}

pub fn timeline_update(timeline: &mut op_engine::Timeline, message: TimelineMessage) {
    match message {
        TimelineMessage::Track(track_number, message) => {
            let track = &mut timeline.tracks[track_number];
            track_update(track, message);
        }
    }
}
