use std::collections::HashMap;
use std::process::exit;
use std::sync::{Arc, Mutex};

use cpal::{FromSample, SizedSample, Stream, StreamConfig};
use cpal::BufferSize::Fixed;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use repl_rs::{Command, Convert, Parameter, Repl, Value};

use op_engine::{Clip, Player, Session};

struct Context {
    session: Arc<Mutex<Session>>,
    player: Arc<Mutex<Player>>,
    stream: Stream,
}

type CommandResult = anyhow::Result<Option<String>>;

fn load(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let mut session = context.session.lock().unwrap();
    session.load_overwrite(&path)?;
    Ok(None)
}

fn save(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let session = context.session.lock().unwrap();
    session.save(&path)?;
    Ok(None)
}

fn place(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let clip_path: String = args["clip_path"].convert()?;
    let pos_sec: f32 = args["pos"].convert()?;
    let track: usize = args["track"].convert()?;
    let mut session = context.session.lock().unwrap();
    let pos_samples = session.sec_to_samples(pos_sec);
    session.add_clip(track, pos_samples, Clip::from_file(&clip_path)?);
    Ok(None)
}

fn export(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let session = context.session.lock().unwrap();
    session.export(&path)?;
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

fn stop(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.stream.pause()?;
    let mut player = context.player.lock().unwrap();
    player.seek(0);
    Ok(None)
}

fn quit(_args: HashMap<String, Value>, _context: &mut Context) -> CommandResult {
    exit(0);
}

fn run<T>(device: &cpal::Device, config: &StreamConfig, player: Arc<Mutex<Player>>) -> anyhow::Result<Stream>
    where
        T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut p = player.lock().unwrap();
            p.write_next_block(data, channels)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn main() -> anyhow::Result<()> {
    let session = Session::new();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("output device required");
    let supported_config = device.default_output_config().expect("output config required");

    let mut config: StreamConfig = supported_config.config();
    config.buffer_size = Fixed(256);

    println!("Output config: {:?}", config);

    if config.sample_rate.0 != session.sample_rate {
        println!("Warning: playback sample rate does not match session sample rate (TODO)")
    }

    let session = Arc::new(Mutex::new(session));
    let player = Arc::new(Mutex::new(Player::new(session.clone())));
    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config, player.clone()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config, player.clone()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config, player.clone()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config, player.clone()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config, player.clone()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config, player.clone()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config, player.clone()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config, player.clone()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config, player.clone()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config, player.clone()),
        sample_format => panic!("unsupported sample format '{sample_format}'"),
    }.expect("run failed");

    let context = Context {
        session,
        player,
        stream,
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
                .with_parameter(Parameter::new("track")
                    .set_required(false)?
                    .set_default("0")?)?
        )
        .add_command(Command::new("play", play))
        .add_command(Command::new("pause", pause))
        .add_command(Command::new("stop", stop))
        .add_command(Command::new("quit", quit));

    repl.run()?;

    Ok(())
}
