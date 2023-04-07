use std::collections::HashMap;
use std::error::Error;
use cpal::{FromSample, Host, Sample, SizedSample, Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use repl_rs::{Command, Convert, Parameter, Repl, Value};
use op_engine::{Clip, Session};

struct Context {
    session: Session,
    stream: Stream,
}

type CommandResult = anyhow::Result<Option<String>>;

fn load(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    context.session = Session::load(&path)?;
    Ok(None)
}

fn save(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    context.session.save(&path)?;
    Ok(None)
}

fn place(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let clip_path: String = args["clip_path"].convert()?;
    let pos_sec: f32 = args["pos"].convert()?;
    let pos_samples = context.session.sec_to_samples(pos_sec);
    context.session.add_clip(0, Clip::from_file(pos_samples, &clip_path)?);
    Ok(None)
}

fn export(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    context.session.export(&path)?;
    Ok(None)
}

fn play(_args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.stream.play()?;
    Ok(None)
}

fn pause(_args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.stream.pause()?;
    Ok(None)
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<Stream, Box<dyn Error>>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    Ok(device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?)
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("output device required");
    let config = device.default_output_config().expect("output config required");
    println!("Default output config: {:?}", config);

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("unsupported sample format '{sample_format}'"),
    }.expect("run failed");

    let context = Context {
        session: Session::new(),
        stream
    };

    context.stream.pause().expect("failed to pause stream");

    let mut repl = Repl::new(context)
        .with_name("op_application tester")
        .add_command(
            Command::new("load", load)
                .with_parameter(Parameter::new("path").set_required(true)?)?
        )
        .add_command(
            Command::new("save", save)
                .with_parameter(Parameter::new("path").set_required(true)?)?
        )
        .add_command(
            Command::new("export", export)
                .with_parameter(Parameter::new("path").set_required(true)?)?
        )
        .add_command(
            Command::new("place", place)
                .with_parameter(Parameter::new("clip_path").set_required(true)?)?
                .with_parameter(Parameter::new("pos").set_required(true)?)?
        )
        .add_command(
            Command::new("play", play)
        )
        .add_command(
            Command::new("pause", pause)
        );

    repl.run()?;

    Ok(())
}
