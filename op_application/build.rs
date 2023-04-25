use std::{env, process};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=engines");

    let temp = env::var("OUT_DIR")?;
    let out_dir = Path::new(&temp);
    let dsp_file = Path::new("engines/sine.dsp");
    let rs_file = out_dir.join("sine.rs");

    let _ = process::Command::new("faust")
        .arg("-wall")
        .arg("-light")
        .args(["-lang", "rust"])
        .args(["-cn", "Sine"])
        .arg(dsp_file)
        .args(["-o", rs_file.to_str().unwrap()])
        .output()?;

    Ok(())
}