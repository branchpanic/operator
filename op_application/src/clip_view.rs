use iced::{Color, Element, Point, Rectangle, Theme};
use iced::widget::Canvas;
use iced::widget::canvas::{Cursor, Frame, Geometry, LineCap, LineJoin, Path, Stroke};
use iced_native::Length;

use op_engine::Clip;

use crate::OpMessage;

pub struct ClipProgram {
    pub clip: Clip,
    pub samples_per_pixel: usize,
}

impl iced::widget::canvas::Program<OpMessage> for ClipProgram {
    type State = ();

    fn draw(&self, _state: &Self::State, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        if self.clip.data.len() > 0 {
            let get_y = |sample: f32| {
                1.0 * (1.0 - sample.abs()) * (bounds.height - 12.0)
            };

            let mut point = Point::ORIGIN;
            let path = Path::new(|builder| {
                builder.move_to(Point::new(bounds.x, get_y(self.clip.data[0])));

                for i in 0..self.clip.data.len() / self.samples_per_pixel {
                    let mut sample = 0.0;

                    for j in 0..self.samples_per_pixel {
                        sample += self.clip.data[i * self.samples_per_pixel + j].abs();
                    }

                    sample /= self.samples_per_pixel as f32;
                    point.x = bounds.x + i as f32;
                    point.y = get_y(sample);
                    builder.line_to(point);
                }

                builder.circle(Point::new(point.x + 5.0, point.y), 5.0);
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
    Canvas::new(ClipProgram { clip, samples_per_pixel: resolution })
        .width(width as f32)
        .height(Length::Fill)
        .into()
}
