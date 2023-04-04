use std::collections::HashMap;
use std::error::Error;
use repl_rs::{Command, Convert, Parameter, Repl, Value};
use op_engine::{Clip, Session};

struct Context {
    session: Session,
}

impl Context {
    fn new() -> Self {
        Context { session: Session::new() }
    }
}

fn load(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>, Box<dyn Error>> {
    let path: String = args["path"].convert()?;
    context.session = Session::load(&path)?;
    Ok(None)
}

fn save(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>, Box<dyn Error>> {
    let path: String = args["path"].convert()?;
    context.session.save(&path)?;
    Ok(None)
}

fn place(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>, Box<dyn Error>> {
    let clip_path: String = args["clip_path"].convert()?;
    let pos_sec: f32 = args["pos"].convert()?;
    let pos_samples = context.session.sec_to_samples(pos_sec);
    context.session.add_clip(0, Clip::from_file(pos_samples, &clip_path)?);
    Ok(None)
}

fn export(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>, Box<dyn Error>> {
    let path: String = args["path"].convert()?;
    context.session.export(&path)?;
    Ok(None)
}

fn main() -> repl_rs::Result<()> {
    let mut repl = Repl::new(Context::new())
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
        );

    repl.run()
}
