use iced::{Element, Length};
use iced::alignment::Vertical;
use iced_native::row;
use iced_native::widget::{column, container, row, scrollable, text};

use op_engine::{Timeline, Track};

use crate::{clip_view, OpMessage};

const BASE_SAMPLES_PER_PIXEL: i32 = 300;

fn track_view(number: usize, track: &Track, zoom: f32) -> Element<'static, OpMessage> {
    let clip_area = row(
        track.iter_clips().map(|clip_inst| {
            clip_view::clip_view(
                clip_inst.clip.clone(),
                (zoom * BASE_SAMPLES_PER_PIXEL as f32) as usize,
            )
        }).collect()
    ).spacing(0).width(Length::Fill);

    let track_header = text(format!("{}", number))
        .height(Length::Fill)
        .vertical_alignment(Vertical::Center);

    row![track_header, clip_area]
        .padding(20.0)
        .spacing(15.0)
        .height(Length::Fill)
        .into()
}

pub fn timeline_view(timeline: &Timeline, zoom: f32) -> Element<'static, OpMessage> {
    container(column(timeline.tracks.iter().enumerate().map(|(i, track)| { track_view(i, track, zoom) }).collect()))
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}