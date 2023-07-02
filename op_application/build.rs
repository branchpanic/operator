use std::{env, process};
use std::path::Path;

fn capitalize_first(s: &str) -> String {
    s.chars().take(1).map(|c| c.to_uppercase().next().unwrap()).chain(s.chars().skip(1)).collect()
}

fn get_class_name(file_name: &str) -> String {
    file_name.split('_').map(capitalize_first).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=engines");

    let temp = env::var("OUT_DIR")?;
    let out_dir = Path::new(&temp);
    let engines_dir = Path::new("engines");

    for dsp_file in engines_dir.read_dir()? {
        if let Err(..) = dsp_file {
            continue;
        }

        let dsp_path = dsp_file.unwrap().path();
        let dsp_name = dsp_path.file_stem().unwrap();
        let rs_path = out_dir.join(dsp_name).with_extension("rs");
        let result = process::Command::new("faust")
            .arg("-wall")
            .arg("-light")
            .args(["-lang", "rust"])
            .args(["-cn", &get_class_name(dsp_name.to_str().unwrap())])
            .arg(dsp_path)
            .args(["-o", rs_path.to_str().unwrap()])
            .output()?;

        assert!(result.status.success());
    }

    Ok(())
}