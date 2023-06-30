use iced::{Color, Element, Point, Rectangle, Theme};
use iced::widget::Canvas;
use iced::widget::canvas::{Cursor, Frame, Geometry, LineCap, LineJoin, Path, Stroke};
use iced_native::Length;

use op_engine::Clip;

use crate::OpMessage;

pub struct ClipProgram {
    clip: Clip,
    resolution: usize,
}

impl iced::widget::canvas::Program<OpMessage> for ClipProgram {
    type State = ();

    fn draw(&self, _state: &Self::State, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        if self.clip.data.len() > 0 {
            let get_y = |sample: f32| {
                (1.0 - sample.abs()) * (0.8 * bounds.height) + 0.1
            };

            let mut point = Point::ORIGIN;
            let path = Path::new(|builder| {
                builder.move_to(Point::new(0.0, get_y(self.clip.data[0])));

                for i in 0..self.clip.data.len() / self.resolution {
                    let mut sample = 0.0;

                    for j in 0..self.resolution {
                        sample += self.clip.data[i * self.resolution + j].abs();
                    }

                    sample /= self.resolution as f32;
                    point.x = i as f32;
                    point.y = get_y(sample);
                    builder.line_to(point);
                }

                // builder.circle(point, 3.0);
            });

            frame.stroke(&path, Stroke::default()
                .with_width(2.0)
                .with_color(Color::WHITE)
                .with_line_cap(LineCap::Square)
                .with_line_join(LineJoin::Bevel));
        }

        vec![frame.into_geometry()]
    }
}

pub fn clip_view(clip: Clip, resolution: usize) -> Element<'static, OpMessage> {
    let width = clip.data.len() / resolution;
    Canvas::new(ClipProgram { clip, resolution })
        .width(width as f32)
        .height(Length::Fill)
        .into()
}

pub fn empty_clip_view(length: usize, resolution: usize) -> Element<'static, OpMessage> {
    let width = length / resolution;
    Canvas::new(ClipProgram { clip: Clip::new(vec![0.0; length]), resolution })
        .width(width as f32)
        .height(Length::Fill)
        .into()
}