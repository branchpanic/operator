use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use cpal::{BufferSize, ChannelCount, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::{Player, Time};

use crate::player::PlayerError;
use crate::project::Project;

/// A Session is a loaded Project plus a context for playing and recording audio.
pub struct Session {
    pub project: Arc<Mutex<Project>>,
    player: Arc<Mutex<Player>>,

    output_stream: cpal::Stream,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("failed to build stream")]
    BuildStreamFailed(#[from] cpal::BuildStreamError),

    #[error(transparent)]
    PlayStreamFailed(#[from] cpal::PlayStreamError),

    #[error(transparent)]
    PauseStreamFailed(#[from] cpal::PauseStreamError),

    #[error(transparent)]
    PlayerError(#[from] PlayerError),
}

fn stream_error_callback(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn build_output_stream<T>(device: &cpal::Device, config: &StreamConfig, player: Arc<Mutex<Player>>) -> Result<cpal::Stream, SessionError>
    where
        T: cpal::SizedSample + cpal::FromSample<f32> + Debug,
{
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut p = player.lock().unwrap();
            p.write_next_block(data, channels)
        },
        stream_error_callback,
        None,
    )?;

    Ok(stream)
}

impl Session {
    pub fn empty_with_defaults() -> Result<Self, SessionError> {
        let project = Arc::new(Mutex::new(Project::new()));

        // TODO: Hosts and devices will eventually need to be configurable.
        let host = cpal::default_host();
        let output_device = host.default_output_device().expect("output device required");

        let output_supported_config = output_device
            .default_output_config()
            .expect("default config for host-provided default output device must be valid");

        // TODO: Validate that buffer size is supported.
        let buffer_size = BufferSize::Fixed(128);
        let output_sample_format = output_supported_config.sample_format();
        println!("Supported config: {:?}", output_supported_config);

        let mut output_config: StreamConfig = output_supported_config.into();
        output_config.buffer_size = buffer_size.clone();

        let player = Arc::new(Mutex::new(Player::new(project.clone(), output_config.clone())?));
        let output_stream;

        {
            use cpal::SampleFormat::*;
            let player_ref = player.clone();
            output_stream = match output_sample_format {
                I8 => build_output_stream::<i8>(&output_device, &output_config, player_ref),
                I16 => build_output_stream::<i16>(&output_device, &output_config, player_ref),
                I32 => build_output_stream::<i32>(&output_device, &output_config, player_ref),
                I64 => build_output_stream::<i64>(&output_device, &output_config, player_ref),
                U8 => build_output_stream::<u8>(&output_device, &output_config, player_ref),
                U16 => build_output_stream::<u16>(&output_device, &output_config, player_ref),
                U32 => build_output_stream::<u32>(&output_device, &output_config, player_ref),
                U64 => build_output_stream::<u64>(&output_device, &output_config, player_ref),
                F32 => build_output_stream::<f32>(&output_device, &output_config, player_ref),
                F64 => build_output_stream::<f64>(&output_device, &output_config, player_ref),
                f => panic!("unsupported sample format '{}'", f),
            }?;
        }

        output_stream.play().expect("could not start output stream");

        println!("Session information:");
        println!("  Output: {}\n    {:?}", output_device.name().unwrap_or("<error>".to_string()), output_config);

        let session = Session {
            project,
            player,
            output_stream,
        };

        Ok(session)
    }

    pub fn play(&mut self) -> Result<(), SessionError> {
        let mut player = self.player.lock().unwrap();
        player.playing_project = true;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), SessionError> {
        let mut player = self.player.lock().unwrap();
        player.playing_project = false;
        Ok(())
    }

    pub fn seek(&mut self, time: Time) {
        let mut player = self.player.lock().unwrap();
        player.seek(time);
    }

    pub fn time(&self) -> Time {
        let player = self.player.lock().unwrap();
        player.time()
    }

    pub fn set_recording(&self, recording: bool, record_track: usize) {
        let mut player = self.player.lock().unwrap();
        player.set_recording(recording, record_track);
    }

    pub fn handle(&self, msg: midly::MidiMessage) {
        let mut project = self.project.lock().unwrap();
        project.generator.handle(msg);
    }
}
