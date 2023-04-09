use std::collections::HashMap;
use std::process::exit;
use std::sync::{Arc, Mutex};

use cpal::{FromSample, SizedSample, Stream, StreamConfig};
use cpal::BufferSize::Fixed;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use repl_rs::{Command, Convert, Parameter, Repl, Value};

use op_engine::{Clip, Player, Project, Session};

struct Context {
    session: Session,
}

type CommandResult = anyhow::Result<Option<String>>;

fn load(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let mut project = context.session.project.lock().unwrap();
    project.load_overwrite(&path)?;
    Ok(None)
}

fn save(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let session = context.session.project.lock().unwrap();
    session.save(&path)?;
    Ok(None)
}

fn place(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let clip_path: String = args["clip_path"].convert()?;
    let pos_sec: f32 = args["pos"].convert()?;
    let track: usize = args["track"].convert()?;
    let mut session = context.session.project.lock().unwrap();
    let pos_samples = session.sec_to_samples(pos_sec);
    session.add_clip(track, pos_samples, Clip::from_file(&clip_path)?);
    Ok(None)
}

fn export(args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    let path: String = args["path"].convert()?;
    let session = context.session.project.lock().unwrap();
    session.export(&path)?;
    Ok(None)
}

fn play(_args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.session.play()?;
    Ok(None)
}

fn pause(_args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.session.pause()?;
    Ok(None)
}

fn stop(_args: HashMap<String, Value>, context: &mut Context) -> CommandResult {
    context.session.pause()?;
    context.session.seek(0);
    Ok(None)
}

fn quit(_args: HashMap<String, Value>, _context: &mut Context) -> CommandResult {
    exit(0);
}

fn main() -> anyhow::Result<()> {
    let session = Session::empty_with_defaults()?;
    let context = Context { session };

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
        .add_command(Command::new("quit", quit))
        .add_command(Command::new("exit", quit));

    repl.run()?;

    Ok(())
}
